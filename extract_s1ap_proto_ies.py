import sys

def main():
    asn_file = sys.argv[1]
    rs_file = sys.argv[2]

    proto_ies = parse_asn_spec(asn_file)   
           
    replace_old_rust(rs_file, proto_ies)

def replace_old_rust(rs_file, proto_ies):
    new_rust = gen_new_rust(proto_ies)

    with open(rs_file, 'r') as infile:
        rs_lines = infile.readlines()

    for typename,_ in proto_ies:
        print(f'replacing {typename}...')

        ies_impl = f'impl entropic::Entropic for {typename}ProtocolIEs '

        line_idx = 0
        found = False
        while line_idx < len(rs_lines):
            if rs_lines[line_idx].find(ies_impl) >= 0:
                while rs_lines[line_idx] != '}\n':
                    rs_lines.pop(line_idx)
                rs_lines.pop(line_idx)
                found = True
                break
            line_idx += 1

        if not found:
            print(f'WARNING: IE "{ies_impl}" not found')

    print('Writing output...')

    with open(rs_file, 'w') as outfile:
        outfile.writelines(rs_lines)
        outfile.write(new_rust)
    
    print('All done!')

def gen_new_rust(proto_ies):
    output = ''

    for typename,fields in proto_ies:
        ies_ty = typename + 'ProtocolIEs'
        ies_entry_ty = ies_ty + '_Entry'
        ies_entryvalue_ty = ies_entry_ty + 'Value'

        from_entropy_fields_output = ''
        to_entropy_fields_output = ''

        for ident,ty,presence in fields:

        
            if presence == 'mandatory':
                from_entropy_fields_output += \
                f'''
    let b = source.get_byte()?;
    if (b & 0b_0001_1111) != 0b_0001_1111 {{ // 1/32 chance of missing
        let ie_value = {ies_entryvalue_ty}::{ident}(source.get_entropic()?);
        ie_list.push({ies_entry_ty} {{
            id: ProtocolIE_ID(ie_value.choice_key()),
            criticality: Criticality(Criticality::IGNORE),
            value: ie_value,
        }});
    }}
                '''
                to_entropy_fields_output += \
                f'''
        if let Some({ies_entryvalue_ty}::{ident}(value)) = self.0.get(ie_idx).map(|ie| &ie.value) {{
            ie_idx += 1;
            length += sink.put_byte(0b_0000_0000)?;
            sink.put_entropic(value)?;
        }} else {{
            length += sink.put_byte(0b_0001_1111)?;
        }};
                '''
            elif presence == 'conditional':
                from_entropy_fields_output += \
                f'''
    let b = source.get_byte()?;
    if (b & 0b_0000_0011) == 0b_0000_0011 {{ // 1/4 chance of being present
        let ie_value = {ies_entryvalue_ty}::{ident}(source.get_entropic()?);
        ie_list.push({ies_entry_ty} {{
            id: ProtocolIE_ID(ie_value.choice_key()),
            criticality: Criticality(Criticality::IGNORE),
            value: ie_value,
        }});
    }}
                '''
                to_entropy_fields_output += \
                f'''
        if let Some({ies_entryvalue_ty}::{ident}(value)) = self.0.get(ie_idx).map(|ie| &ie.value) {{
            ie_idx += 1;
            length += sink.put_byte(0b_0000_0011)?;
            sink.put_entropic(value)?;
        }} else {{
            length += sink.put_byte(0b_0000_0000)?;
        }};
                '''
            elif presence == 'optional':
                from_entropy_fields_output += \
                f'''
    let b = source.get_byte()?;
    if (b & 0b_0000_1111) == 0b_0000_1111 {{ // 1/16 chance of being present
        let ie_value = {ies_entryvalue_ty}::{ident}(source.get_entropic()?);
        ie_list.push({ies_entry_ty} {{
            id: ProtocolIE_ID(ie_value.choice_key()),
            criticality: Criticality(Criticality::IGNORE),
            value: ie_value,
        }});
    }}
                '''
                to_entropy_fields_output += \
                f'''
        if let Some({ies_entryvalue_ty}::{ident}(value)) = self.0.get(ie_idx).map(|ie| &ie.value) {{
            ie_idx += 1;
            length += sink.put_byte(0b_0000_1111)?;
            sink.put_entropic(value)?;
        }} else {{
            length += sink.put_byte(0b_0000_0000)?;
        }};
                '''
            else:
                sys.exit(f'Unknown presence value {presence}')
        
        output += f'''
impl entropic::Entropic for {ies_ty} {{
    #[inline]
    fn from_entropy_source<'a, I: Iterator<Item = &'a u8>, E: EntropyScheme>(
        source: &mut Source<'a, I, E>,
    ) -> Result<Self, Error> {{
        let mut ie_list = Vec::new();

        // Loop this part for every enum discriminant

        {from_entropy_fields_output}
        
        Ok({ies_ty}(ie_list))
    }}

    #[inline]
    fn to_entropy_sink<'a, I: Iterator<Item = &'a mut u8>, E: EntropyScheme>(
        &self,
        sink: &mut Sink<'a, I, E>,
    ) -> Result<usize, Error> {{
        let mut ie_idx = 0;
        let mut length = 0;

        {to_entropy_fields_output}

        if ie_idx != self.0.len() {{
            return Err(entropic::Error::Internal)
        }}

        Ok(length)
    }}
}}
    '''

    return output





def parse_asn_spec(asn_file):
    with open(asn_file, 'r') as infile:
        asn_spec = infile.read()

    proto_ies_idx = 0

    proto_ies = []

    while True:
        proto_ies_idx = asn_spec.find(" S1AP-PROTOCOL-IES ::= {", proto_ies_idx)
        if proto_ies_idx < 0:
            break
        
        # get the name of the type
        type_name_idx = proto_ies_idx
        while asn_spec[type_name_idx - 1] != '\n':
            type_name_idx -= 1

        typename = asn_spec[type_name_idx:proto_ies_idx]
        typename = typename[:-3].replace('-', '_')
        if typename[-1] == '_':
            typename = typename[:-1]


        proto_ies_idx += len(" S1AP-PROTOCOL-IES ::= {")

        # Make sure the type conforms to name requirements (not robust yet)
        if not is_valid_typename(typename):
            print(f'typename "{typename}" is not valid')
            continue



        # Parse all IE fields
        fields = []
        while True:
            if asn_spec[proto_ies_idx].isspace() or asn_spec[proto_ies_idx] == '|':
                proto_ies_idx += 1
                continue
            if asn_spec[proto_ies_idx] == ',':
                proto_ies_idx += 1
                while asn_spec[proto_ies_idx].isspace():
                    proto_ies_idx += 1
                if asn_spec[proto_ies_idx:proto_ies_idx + 3] != '...':
                    print(f"WARNING: unexpected missing ' ...' from end of Protocol IEs def: {asn_spec[proto_ies_idx:proto_ies_idx+20]}")
                proto_ies_idx += 3
                break # We don't need to parse the closing '}'; we're just grepping anyways.
 
            while asn_spec[proto_ies_idx].isspace():
                proto_ies_idx += 1

            if asn_spec[proto_ies_idx:proto_ies_idx+2] == '--':
                proto_ies_idx = asn_spec.index('\n', proto_ies_idx) + 1
                while asn_spec[proto_ies_idx].isspace():
                    proto_ies_idx += 1               

            if asn_spec[proto_ies_idx] != '{':
                sys.exit(f"ERROR: missing Protocol IEs field definition at {proto_ies_idx}")

            f_start = proto_ies_idx
            proto_ies_idx = asn_spec.index('}', proto_ies_idx)
            proto_field = asn_spec[f_start:proto_ies_idx]
            fields.append(parse_proto_field(proto_field))
            
            proto_ies_idx += 1

        proto_ies.append((typename,fields))

    return proto_ies

def parse_proto_field(f):
    i = 0
    while f[i].isspace():
        i += 1

    assert(f[i] == '{')
    i += 1

    while f[i].isspace():
        i += 1   

    assert(f[i:i+3] == 'ID ')
    i += 3

    id_start = i
    i = f.index('CRITICALITY', i)

    ident = f[id_start:i].strip()

    i = f.index('TYPE ', i)
    i += 5
    ty_start = i

    i = f.index('PRESENCE', i)

    ty = f[ty_start:i].strip()
    i += len('PRESENCE')

    while f[i].isspace():
        i += 1

    presence_end = i
    while presence_end < len(f) and f[presence_end].isalpha():
        presence_end += 1
    
    presence = f[i:presence_end]

    ident = ident.replace('-', '_')
    if ord(ident[0]) >= ord('a') and ord(ident[0]) <= ord('z'):
        ident = chr(ord('A') + ord(ident[0]) - ord('a')) + ident[1:]
    return (ident, ty, presence)


def is_valid_typename(typename):
    return True
#    for c in typename:
#        if c != '-' and (not (ord(c) >= ord('a') and ord(c) <= ord('z'))) and (not (ord(c) >= ord('A') and ord(c) <= ord('Z'))) and (not (ord(c) >= ord('0') and ord(c) <= ord('9'))):
#            return False
#    return True


if __name__ == '__main__':
    main()

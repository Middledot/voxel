"""
Python script to auto-generate the readers/writers for the MCBE protocol
(Au)tomatically (G)enerated (B)edrock (E)dition (P)rotocol

Will probably port to rust eventually
"""


import orjson

# level1_types = []
# for k, _ in data.items():
#     level1_types.append(k)
# print(level1_types)


## Stuff to do
# 1. Make MsgBuffer work with the native datatypes (new ones that need to be implemented)
# 2. Structure of a packet file:
"""
use crate::... // ToBuffer, FromBuffer, MsgBuffer
<dependency1.outer>
<dependency2.inner>  // use super::..

pub struct <CamelCasePacket> {
    pub <orderedItem1>: type1;
    pub <orderedItem2>: type2;
    pub <orderedItem3>: type3;
}

impl PacketID for <CamelCasePacket> {
    const ID: u8 = <CamelCasePacket.ID>
}

impl FromBuffer for <CamelCasePacket> {
    fn from_buffer(buf: MsgBuffer) -> Self {
        let <orderedItem1>: <type1> = <type1{op=read}>  // uses <dependency1.outer>
        let <orderedItem2>: <type2> = <type2{op=read}>
        let <orderedItem3>: <type3> = <type3{op=from_buffer(&buf)}>  // uses <dependency2.inner> (i.e. other datatype)

        Self {
            <orderedItem1>,
            <orderedItem2>,
            <orderedItem3>,
        }
    }
}

impl ToBuffer for <CamelCasePacket> {
    fn to_buffer(&self) -> MsgBuffer {
        buf = MsgBuffer::new();
        <type1{op=write}>  // uses <dependency1.outer>
        <type2{op=write}>
        <type3{op=to_buffer}>  // uses <dependency2.inner> (i.e. other datatype)

        buf
    }
}

"""
# of course, we can manually write exceptions to this standard
# ALSO, enums will be separate files, need special checks for naming
# same with bitflags



class Command:
    pass
    # contains stuff like datatype, strings for reading/writing, dependencies


class GenericIOObject:
    @classmethod
    def from_data(packet_id: int, blueprint: dict):
        self = GenericIOObject()

        self.packet_id = packet_id
        self.blueprint = blueprint


class AugbepParser:
    def __init__(self):
        self.packet_blueprints = {}
        self.packet_blueprint_mappings = None

        self.map_packetid_to_packetname: dict[str, int] = {}

        self.genericio = {}

    def handle_native(key: str):
        match key:
            case "varint64":
                print("buf.{op}_i64_varint_bytes")
            case "zigzag32":
                print("buf.{op}_zigzag32")
            case "zigzag64":
                print("buf.{op}_zigzag64")
            # case "uuid":
            #     # TODO:
            # case "byterot":
            #     # TODO: what in tarnation is this?
            # case "bitflags":
            #     # TODO: 

    def parse(self):
        data = orjson.loads(open("protocol.json", "rb").read())["types"]

        # Example thing
        # "packet_request_network_settings": [
        #     "container",
        #     [
        #         {
        #             "name": "client_protocol",
        #             "type": "i32"
        #         }
        #     ]
        # ],

        for k, v in data.items():
            k: str
            if k == "mcpe_packet":
                self.packet_mapping = v
                self.map_packetid_to_packetname = {v2: k2 for k2, v2 in v.items()}
            elif isinstance(v, str):
                print(k)
            elif k.startswith("packet") and v[0] == "container":
                self.packet_blueprint_mappings[k.replace("packet_", "", 1)] = v[1]

        for name, blueprint in self.packet_blueprint_mappings.items():
            self.genericio.append(
                GenericIOObject.from_data(
                    self.map_packetid_to_packetname[name],
                )
            )


        #     if isinstance(v, list) and v[0] not in lvl2types:
        #         lvl2types.append(v[0])

        # print(lvl2types)

AugbepParser().parse()

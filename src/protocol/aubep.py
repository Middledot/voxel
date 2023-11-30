"""
Python script to auto-generate the readers/writers for the MCBE protocol
(Au)tomatic (Be)drock (P)rotocol (pronounced aw-bep)

Will probably port to rust eventually
"""


import orjson, textwrap, abc
from enum import Enum
from collections import OrderedDict

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

def name_camel_case(name):
    if name.startswith("_"):
        name = name[1:]
    return ''.join(x.capitalize() for x in name.split('_'))

def get_core_type(type):
    match type.lstrip("l"):
        case "u8":
            return u8
        case "bool":
            return Bool
        case _:
            return Num

def get_type(param):
    if (t := get_core_type(param["type"])):
        if issubclass(t, Num):
            return t(param["name"], param["type"])
        # print("this", t, isinstance(t, Num))
        return t(param["name"])


class Endianess(Enum):
    Big = 0
    Little = 1


class Property:
    def __init__(self, name, type):
        self.abs_deps = []
        self.deps = []

        if name in ["type"]:
            name = f"_{name}"
        self.name = name
        self.type = type  # replace with type objects (w/ dependencies and such)

    @property
    def name_camel_case(self):
        return name_camel_case(self.name)

    def attr(self):
        return "pub {0.name}: {0.type},".format(self)

    @abc.abstractmethod
    def to_buffer(self) -> str:
        pass
    
    @abc.abstractmethod
    def from_buffer(self) -> str:
        pass


class Num(Property):
    def __init__(self, name: str, type: str):
        if type.startswith("l"):
            self.endianess = Endianess.Little
        else:
            self.endianess = Endianess.Big

        super().__init__(name, type.lstrip("l"))

    def from_buffer(self) -> str:
        return "let {0} = buf.read_{1}_{2}_bytes();".format(self.name, self.type, self.get_endianess_abbr())

    def to_buffer(self) -> str:
        return "buf.write_{1}_{2}_bytes(self.{0});".format(self.name, self.type, self.get_endianess_abbr())

    def get_endianess_abbr(self):
        match self.endianess:
            case Endianess.Little:
                return "le"
            case Endianess.Big:
                return "be"


class Mapper(Property):
    # TODO:
    def __init__(self, name, core: dict):
        super().__init__(name, name_camel_case(name))
        self.inner_type = get_core_type(core["type"])

        self.items = OrderedDict(
            sorted(
                {
                    int(num): self.inner_type(name, core["type"])
                    for num, name in core["mappings"].items()
                }.items(),
                key=lambda o: o[0],
            )
        )
    
    def from_int(self):
        if len(self.items) == 0:
            return ""

        finalstr = "\n        match value {\n"
        contents = ""

        for ind, prop in self.items.items():
            contents += "{0} => {1}::{2},\n".format(ind, self.name_camel_case, prop.name_camel_case)

        finalstr += textwrap.indent(contents, "            ")
        finalstr += "        }\n    "

        return finalstr

    def attrs(self):
        finalstr = ""
        for ind, attr in self.items.items():
            finalstr += attr.name_camel_case + " = " + str(ind) + ",\n"

        return textwrap.indent(finalstr, "    ")

    def from_buffer(self):
        # my names hacker, pro hacker
        datatype = next(iter(self.items.values()))
        return "let {0.name} = {0.name_camel_case}::from_{2}({1});".format(self, datatype.from_buffer().split(" = ")[1].rstrip(";"), self.inner_type.type)

    def to_buffer(self):
        datatype = next(iter(self.items.values()))
        return "{1}(self.{0} as {2});".format(self.name, datatype.to_buffer().split("(")[0], self.inner_type.type)


class u8(Num):
    def __init__(self, name, type):
        super().__init__(name, type)

    def from_buffer(self) -> str:
        return "let {} = buf.read_byte();".format(self.name)
    
    def to_buffer(self) -> str:
        return "buf.write_byte(self.{});".format(self.name)


class i32(Num):
    def __init__(self, name, type):
        super().__init__(name, type)


class u16(Num):
    def __init__(self, name: str, type):
        super().__init__(name, type)


class f32(Num):
    def __init__(self, name, type):
        super().__init__(name, type)


class Bool(Property):
    def __init__(self, name):
        super().__init__(name, "bool")

    def from_buffer(self):
        return "let {} = buf.read_byte() != 0;".format(self.name)

    def to_buffer(self):
        return "buf.write_byte(self.{} as u8);".format(self.name)


class GenericIOObject:
    def __init__(self, packet_id, name, props):
        self.abs_deps = ["use crate::raknet::objects::MsgBuffer;", "use crate::raknet::packets::{ToBuffer, FromBuffer, PacketID};"]
        self.deps = []

        self.packet_id = packet_id
        self.name: str = name

        self.props: list[Property] = props
    
    @property
    def mappers(self):
        mappers = []
        for prop in self.props:
            if isinstance(prop, Mapper):
                mappers.append(prop)

        if not mappers:
            return ""
        
        finalstr = (
            "\n"
            "pub enum {0.name_camel_case} {{\n"
            "{1}"
            "}}\n"
            "\n"
            "impl {0.name_camel_case} {{\n"
            "    pub fn from_{0.inner_type.type}(value: {0.inner_type.type}) -> Self {{"
            "{2}"
            "}}\n"
            "}}\n"
            "\n"
        ).format(mappers[0], mappers[0].attrs(), mappers[0].from_int())

        return finalstr

    @property
    def name_camel_case(self):
        return ''.join(x.capitalize() for x in self.name.split('_'))

    @property
    def fmt_packet_id(self):
        return f"{hex(self.packet_id)};  // {self.packet_id}"

    @property
    def all_deps(self):
        deps = self.abs_deps
        for p in self.props:
            for ad in p.abs_deps:
                if ad not in deps:
                    deps.append(ad)
            for d in p.deps:
                if d not in deps:
                    deps.append(d)

        return '\n'.join(deps)

    def attrs(self):
        finalstr = ""
        for prop in self.props:
            finalstr += prop.attr() + "\n"
        
        return textwrap.indent(finalstr, "    ")

    def from_buffer(self):
        obj_ret = "Self {"
        finalstr = ""
        for prop in self.props:
            obj_ret += "\n    " + prop.name + ","
            finalstr += prop.from_buffer() + "\n"
        obj_ret += "\n}"
        finalstr += "\n" + obj_ret

        return textwrap.indent(finalstr, "        ")

    def to_buffer(self):
        finalstr = ""
        for prop in self.props:
            finalstr += prop.to_buffer() + "\n"

        return textwrap.indent(finalstr, "        ")

    def format(self):
        return (
            "/* Auto-generated by augbep */\n"
            "{0.all_deps}\n"
            "{4}"
            "\n"
            "pub struct {0.name_camel_case} {{\n"
            "{1}"
            "}}\n"
            "\n"
            "impl PacketID for {0.name_camel_case} {{\n"
            "    const ID: u8 = {0.fmt_packet_id}\n"
            "}}\n"
            "\n"
            "impl FromBuffer for {0.name_camel_case} {{\n"
            "    fn from_buffer(buf: &mut MsgBuffer) -> Self {{\n"
            "{2}\n"
            "    }}\n"
            "}}\n"
            "\n"
            "impl ToBuffer for {0.name_camel_case} {{\n"
            "    fn to_buffer(&self) -> MsgBuffer {{\n"
            "        let mut buf = MsgBuffer::new();\n"
            "{3}"
            "\n"
            "        buf\n"
            "    }}\n"
            "}}\n"
        ).format(self, self.attrs(), self.from_buffer(), self.to_buffer(), self.mappers)  #, self.from_buffer(), self.to_buffer())

    @classmethod
    def from_data(cls, packet_id: int, name: str, blueprint: dict):
        props = []
        for param in blueprint:
            if not param.get("type"):
                continue

            if isinstance(param["type"], list) and param["type"][0] == "mapper":
                props.append(Mapper(param["name"], param["type"][1]))
            elif (prop := get_type(param)):
                props.append(prop)
            else:
                props.append(
                    Property(**param)
                )

        return cls(packet_id, name, props)


class AugbepParser:
    def __init__(self):
        self.folder = "./v622/"
        self.packet_blueprints = {}
        self.packet_blueprint_mappings = {}

        self.map_packetid_to_packetname: dict[str, int] = {}

        self.genericio: list[GenericIOObject] = []

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
        data: dict = orjson.loads(open("protocol.json", "rb").read())["types"]

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
                self.map_packetid_to_packetname = {v2: int(k2) for k2, v2 in v[1][0]["type"][1]["mappings"].items()}
                # print(self.map_packetid_to_packetname)
            elif isinstance(v, str):
                print(k)
            elif k.startswith("packet") and v[0] == "container":
                # print(k.replace("packet_", "", 1))
                # print(v)
                self.packet_blueprint_mappings[k.replace("packet_", "", 1)] = v[1]


        for name, blueprint in self.packet_blueprint_mappings.items():
            try:
                gen = GenericIOObject.from_data(
                    self.map_packetid_to_packetname[name],
                    name,
                    blueprint
                )
            except Exception as e:
                # print(e)
                pass
            else:
                self.genericio.append(gen)

        for obj in self.genericio:
            var = ""
            try:
                var = (obj.format())
            except Exception as e:
                # print('bruh')
                import traceback
                var = (traceback.format_exc())
            else:
                with open(self.folder + obj.name + ".rs", "w") as fp:
                    fp.write(var)

        with open(self.folder + "mod.rs", "w") as fp:
            for mod in self.genericio:
                fp.write("pub mod " + mod.name + ";\n")
            fp.write("\n")
            for mod in self.genericio:
                fp.write("pub use " + mod.name + "::" + mod.name_camel_case + ";\n")


        #     if isinstance(v, list) and v[0] not in lvl2types:
        #         lvl2types.append(v[0])

        # print(lvl2types)

AugbepParser().parse()

# obj = GenericIOObject.from_data(
#     193,
#     "request_network_settings",
#     [
#         {
#             "name": "client_protocol",
#             "type": "i32"
#         }
#     ]
# )

# print(obj.format())

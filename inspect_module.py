import inspect
import importlib
import pathlib
import sys
import json
import types
import collections.abc
import typing


def type_as_tag(ty: type) -> dict:
    if ty is None or ty == inspect._empty:
        return {"kind": "none"}
    if ty == int:
        return {"kind": "primitive", "name": "int"}
    if ty == str:
        return {"kind": "primitive", "name": "str"}
    if ty == float:
        return {"kind": "primitive", "name": "float"}
    if ty == Exception:
        return {"kind": "exception"}
    if ty == Ellipsis:
        return {"kind": "ellipsis"}
    if type(ty) == types.GenericAlias:
        if ty.__origin__ in [list, collections.abc.Sequence]:
            return {"kind": "list", "inner": [type_as_tag(t) for t in ty.__args__]}
        if ty.__origin__ == tuple:
            tags = [type_as_tag(t) for t in ty.__args__]
            return {"kind": "tuple", "tags": tags}
        if ty.__origin__ == dict:
            tags = [type_as_tag(t) for t in ty.__args__]
            return {"kind": "dict", "inner": tags}
        if ty.__origin__ == type:
            return {"kind": "type", "name": ty.__args__[0].__name__}
    if type(ty) == type:
        return {"kind": "user_defined", "name": ty.__name__}
    if type(ty) == typing.NewType:
        return {
            "kind": "user_defined",
            "module": ty.__module__,
            "name": ty.__name__,
            "supertype": type_as_tag(ty.__supertype__),
        }
    if type(ty) in [types.UnionType, typing._UnionGenericAlias]:
        return {"kind": "union", "args": [type_as_tag(t) for t in ty.__args__]}
    if type(ty) == collections.abc._CallableGenericAlias:
        return {
            "kind": "callable",
            "args": [type_as_tag(t) for t in ty.__args__[:-1]],
            "return": type_as_tag(ty.__args__[-1]),
        }
    raise NotImplementedError(f"Unsupported type = {ty}, {type(ty)}")


def inspect_module(target: str) -> str:
    module = importlib.import_module(target)
    interface = {"functions": {}, "type_definitions": {}}
    for name, attr in inspect.getmembers(module):
        if inspect.isfunction(attr):
            sig = inspect.signature(getattr(module, name))
            interface["functions"][name] = {
                "name": name,
                "parameters": [
                    {"name": name, "type": type_as_tag(p.annotation)}
                    for name, p in sig.parameters.items()
                ],
                "return": type_as_tag(sig.return_annotation),
            }
        if type(attr) == typing.NewType:
            interface["type_definitions"][name] = {
                "module": attr.__module__,
                "name": attr.__name__,
                "supertype": type_as_tag(attr.__supertype__),
            }
    return json.dumps(interface, indent=4)


def main() -> int:
    if len(sys.argv) <= 1:
        print(f"usage: {sys.argv[0]} <your_module.py>", file=sys.stderr)
        return 1
    path = pathlib.Path(sys.argv[1])
    if path.exists():
        # as single file module
        sys.path.append(str(path.parent))
        print(inspect_module(path.stem))
    else:
        # as installed module
        print(inspect_module(str(path)))
    return 0


if __name__ == "__main__":
    sys.exit(main())

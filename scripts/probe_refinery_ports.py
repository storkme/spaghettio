from draftsman.data import entities
for name in ("oil-refinery", "chemical-plant"):
    e = entities.raw[name]
    print(f"== {name}")
    for fb in (e.get("fluid_boxes") or []):
        conns = [(c.get("flow_direction", "?"), c.get("position"), c.get("direction")) for c in fb.get("pipe_connections", [])]
        print("  ", fb.get("production_type"), conns)

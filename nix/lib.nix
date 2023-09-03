{ lib }: {
  cleanSourceWithRel = { src, filter }:
    let
      baseStr = builtins.toString src;
    in
    lib.cleanSourceWith {
      inherit src;
      filter = path: type:
        let
          relPath = lib.removePrefix baseStr (toString path);
        in
        filter relPath type;
    };
}

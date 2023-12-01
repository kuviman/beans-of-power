{
  inputs = {
    geng.url = "github:geng-engine/cargo-geng";
  };
  outputs = { self, geng }: geng.makeFlakeOutputs (system:
    {
      src = ./.;
    });
}

Here is a simplified illustration of how a temporary environment would be built from source.

```mermaid
flowchart
  nix[nix-shell -p hello] --> |evaluates| nixpkgshello
  subgraph store
    nixpkgshello["/nix/store/...-nixpkgs-22.05pre.../pkgs/applications/misc/hello/default.nix"] -->|evaluates to| nixhellodrv["/nix/store/asd...-hello-2.10.drv"]
    nixmake["/nix/store/...-gnumake-4.3/bin/make"] -.-> |make| nixhellodrv
    nixgcc["/nix/store/...-gcc-wrapper-10.3.0/bin/gcc"]  -.-> |gcc| nixhellodrv
    nixhellosrc["/nix/store/qwe...-hello-2.10/"]  -.-> |source| nixhellodrv
    nixhellodrv --> |builds| nixhellobin["/nix/store/zxc...-hello-2.10/bin/hello"]
  end
  nixhellobin --> |"PATH=/nix/store/zxc...-hello-2.10/bin/hello:$PATH"| shell
  subgraph shell
  bash[" > which hello\n/nix/store/zxc...-hello-2.10/bin/hello"]
  end
```

Looking more closely into how a build is preformed, we notice that simple build systems such as `make` reference build inputs by name. This is fine if they are files in the source release, such as `src/hello.c`, as their contents are determined with respect to the `Makefile` referencing them. External names such as `gcc` on the other hand are subject to interpretation by the surrounding environment, and would usually resolve to something like `/usr/bin/gcc`, which is yet another name that does not tell us anything about contents. The Nix standard build environment is prepared such that these names are resolved to exact versions specified in the build plan.

```mermaid
flowchart
  nix[nix-shell -p hello] --> |evaluates| nixpkgshello
  subgraph stdenv["standard environment (stdenv)"]
    make[make]
    subgraph hello["/tmp/...-hello-2.10/"]
      subgraph Makefile
        subgraph target[all:]
            gcc
            c["src/hello.c"]
            h["src/system.h"]
            i[-Ilib]
        end
      end
      c -.-> csrc
      h -.-> hsrc
      i -.-> lib
      csrc["src/hello.c"]
      hsrc["src/system.h"]
      lib["lib/"]
    end
        
  end

  make -->target
  make -.-> nixmake
  gcc -.-> nixgcc
  nixhellosrc  -.-> |copy| hello
  nixhellodrv --> |build plan| stdenv
  stdenv --> nixhellobin

  subgraph store
    nixpkgshello["/nix/store/...-nixpkgs-22.05pre.../pkgs/applications/misc/hello/default.nix"] -->|evaluates to| nixhellodrv
    nixmake["/nix/store/...-gnumake-4.3/bin/make"]
    nixgcc["/nix/store/...-gcc-wrapper-10.3.0/bin/gcc"]
    nixhellosrc["/nix/store/qwe...-hello-2.10/"]
    nixhellodrv["/nix/store/asd...-hello-2.10.drv"]
    nixhellobin["/nix/store/asd...-hello-2.10/bin/hello"]
  end
    nixhellobin --> |"PATH=/nix/store/zxc...-hello-2.10/bin/hello:$PATH"| shell
  subgraph shell
  bash[" > which hello\n/nix/store/zxc...-hello-2.10/bin/hello"]
  end
```
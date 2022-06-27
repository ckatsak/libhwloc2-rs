# hwloc2-sys

## Build `hwloc-2.7.1`

Assuming the parent workspace/repo root resides in `WORKSPACE_ROOT_DIR`:

To build & install `hwloc-2.7.1` (minimally):

```console
$ cd "$WORKSPACE_ROOT_DIR"
$ curl -LO https://download.open-mpi.org/release/hwloc/v2.7/hwloc-2.7.1.tar.gz
$ tar -xzf hwloc-2.7.1.tar.gz
$ cd hwloc-2.7.1
$ mkdir _build
$ ./configure --prefix=$PWD/_build --disable-cairo --disable-libxml2 --disable-io --disable-pci --disable-opencl --disable-cuda --disable-nvml --disable-rsmi --disable-levelzero --disable-gl --disable-libudev --disable-plugin-dlopen --disable-plugin-ltdl
$ make -j$(nproc)  && echo $?
$ make install  && echo $?
```

To also be able to find I/O objects (bridges, PCI & OSDev devices, etc), make
sure `libpciaccess-dev` is installed, and remove the `--disable-io` and `--disable-pci`
flags from the above `./configure`.

Some env vars that I found useful for local development:

```console
$ export LIBHWLOC_ROOT="$WORKSPACE_ROOT_DIR/hwloc-2.7.1/_build/"
$ export PKG_CONFIG_PATH="$LIBHWLOC_ROOT/lib/pkgconfig:$PKG_CONFIG_PATH"
$ export LIBRARY_PATH="$LIBHWLOC_ROOT/lib:$LIBRARY_PATH"
$ export LD_LIBRARY_PATH="$LIBHWLOC_ROOT/lib:$LD_LIBRARY_PATH"
```

## Bindgen

```console
$ cd "$WORKSPACE_ROOT_DIR/hwloc2-sys"
$ bindgen -o src/bindings.rs hwloc-2.7.1/include/hwloc.h -- -Ihwloc-2.7.1/include/
$ bindgen --allowlist-function '.*hwloc.*' --allowlist-type '.*hwloc.*' --allowlist-var '(.*hwloc.*|.*HWLOC.*)' -o bindings.rs hwloc-2.7.1/include/hwloc.h -- -Ihwloc-2.7.1/include/
$ bindgen --no-layout-tests --no-doc-comments --allowlist-function '.*hwloc.*' --allowlist-type '.*hwloc.*' --allowlist-var '(.*hwloc.*|.*HWLOC.*)' -o bindings.rs hwloc-2.7.1/include/hwloc.h -- -Ihwloc-2.7.1/include/
$ bindgen --generate-inline-functions --allowlist-function '.*hwloc.*' --allowlist-type '.*hwloc.*' --allowlist-var '(.*hwloc.*|.*HWLOC.*)' -o bindings.rs hwloc-2.7.1/include/hwloc.h -- -Ihwloc-2.7.1/include/
```

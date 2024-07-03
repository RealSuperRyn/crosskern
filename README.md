# crosskern
Cross OS kernel.
Current supported bootloaders: Limine
Current supported architectures: x86_64

# Building
Clone the repository, and enter it.  
Additionally, clone RealSuperRyn/crosshw, and have it be in the same directory as the crosskern repo.  
Then, assuming you're in the root of the repository (important):  
`cargo +nightly build --target ./x86_64-custom-linker.json -Zbuild-std=core,compiler_builtins --release`  
  
Next, `cd limine_support`  
  
Then, use the shell script `./get_compile_limine.sh` which will:  
- Clone the Limine repository 
- Compile Limine
- Create an ESP directory with the boot files and kernel
# Running
If you have qemu-system-x86_64 available, you can use `limine_support/run.sh` while inside the `limine_support` directory.

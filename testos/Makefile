arch ?= x86_64
xargo-target-file ?= rust-os
rust_os := target/$(xargo-target-file)/debug/libos_main.a
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

linker_script := src/linker.ld
grub_cfg := src/grub.cfg
assembly_source_files := $(wildcard src/*.asm)
assembly_object_files := $(patsubst src/%.asm, \
	build/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -s -S -d guest_errors 

gdb:	
	@~/rust-gdb/rust-os-gdb/bin/rust-gdb ~/rust-gdb/testos/build/kernel-x86_64.bin -ex "target remote :1234"

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles	
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)


# compile assembly files
build/%.o: src/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@ -F dwarf

# compile rust
kernel:
	@./xargo build --target $(xargo-target-file)

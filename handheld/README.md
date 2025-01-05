# ME2 Handheld

## Hardware configuration

The ME2 handheld consists of the following major components

### Flash

The main ME2 firmware is stored on an SST39VF3201, a 2 megaword (4 megabytes; each addressible unit is 16 bits) flash chip. A dump of this flash chip is at `flash.bin` in this directory.

### Microcontroller

The main processor is a GeneralPlus GPL162002A stored under the larger epoxy blob on the board. Its instruction set is µ'nSP. It has an internal ROM that provides some hardware abstraction features and handles the initial boot routine. A dump of the internal rom lovingly named `GPL162002A_embadded_rom.bin` due to it being misspelled in the datasheet. The dump was obtained by exploiting a bug in the ME2's USB flash read command in order to read the ROM's address range. The processor was identified by decapsulating the chip.

### ELAN chip

There is another chip under a smaller epoxy blob. I don't know what it is, but it has ELAN Microelectronics' `*M*ELAN` marking and `EU5221-C26YA` on the silicon die. An image of the die is at `EU5221.jpg`.

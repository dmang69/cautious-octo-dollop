#include <stdint.h>
#include <stddef.h>

// Hardware text mode color constants.
enum vga_color {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_WHITE = 15,
};

static inline uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) {
    return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) {
    return (uint16_t) uc | (uint16_t) color << 8;
}

void kmain(void) {
    // VGA text buffer is located at physical address 0xB8000
    uint16_t* terminal_buffer = (uint16_t*) 0xB8000;
    
    // Clear the screen (80 columns by 25 rows)
    uint8_t color = vga_entry_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    for (int y = 0; y < 25; y++) {
        for (int x = 0; x < 80; x++) {
            const size_t index = y * 80 + x;
            terminal_buffer[index] = vga_entry(' ', color);
        }
    }

    // Print a welcome message
    const char* str = "IntentKernel v1.1.0 - Boot Stage 1 Successful!";
    for (size_t i = 0; str[i] != '\0'; i++) {
        terminal_buffer[i] = vga_entry(str[i], color);
    }

    // Halt the CPU
    while (1) {
        __asm__ __volatile__("hlt");
    }
}
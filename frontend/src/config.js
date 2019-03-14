/* eslint-disable no-bitwise */

export const width = process.env.CANVAS_WIDTH;
export const height = process.env.CANVAS_HEIGHT;

export const palette = [
  0xffffff, // white
  0xe4e4e4, // light grey
  0x888888, // gray
  0x222222, // black
  0xffa7d1, // pink
  0xe50000, // red
  0xe59500, // orange
  0xa06a42, // brown
  0xe5d900, // yellow
  0x94e044, // lime
  0x02be01, // green
  0x00d3dd, // cyan
  0x0083c7, // blue
  0x0000ea, // dark blue
  0xcf6ee4, // magenta
  0x820080, // purple
].map(hex => [(hex >> 16) & 0xff, (hex >> 8) & 0xff, hex & 0xff]);

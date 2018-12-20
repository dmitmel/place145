export const width = process.env.CANVAS_WIDTH;
export const height = process.env.CANVAS_HEIGHT;

function color(hex) {
  // eslint-disable-next-line no-bitwise
  return [2, 1, 0].map(i => (hex >> (8 * i)) & 0xff);
}

export const palette = [
  color(0xffffff), // white
  color(0xe4e4e4), // light grey
  color(0x888888), // gray
  color(0x222222), // black
  color(0xffa7d1), // pink
  color(0xe50000), // red
  color(0xe59500), // orange
  color(0xa06a42), // brown
  color(0xe5d900), // yellow
  color(0x94e044), // lime
  color(0x02be01), // green
  color(0x00d3dd), // cyan
  color(0x0083c7), // blue
  color(0x0000ea), // dark blue
  color(0xcf6ee4), // magenta
  color(0x820080), // purple
];

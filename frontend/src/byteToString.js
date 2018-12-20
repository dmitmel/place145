export default function byteToString(byte) {
  return `0${byte.toString(16)}`.slice(-2);
}

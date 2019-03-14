import React from 'react';
import { width, height, palette } from './config';
import CanvasRenderer from './CanvasRenderer';
import PanZoom from './PanZoom';
import ColorPicker from './ColorPicker';
import './App.scss';

function packetToString(packet) {
  return Array.prototype.map
    .call(packet, byte => `0${byte.toString(16)}`.slice(-2))
    .join(' ');
}

export default class App extends React.Component {
  state = { selectedColor: Math.floor(Math.random() * palette.length) };

  canvasRef = React.createRef();

  componentDidMount() {
    this.fetchEntireCanvas();
    this.ws = this.connectToWebsocketAPI();
  }

  componentWillUnmount() {
    this.ws.close();
  }

  onCanvasClick = (x, y) => {
    const { selectedColor } = this.state;
    this.sendSetCell(x, y, selectedColor);
  };

  onColorSelected = color => this.setState({ selectedColor: color });

  fetchEntireCanvas() {
    fetch('/api/canvas')
      .then(response => response.arrayBuffer())
      .then(arrayBuffer => {
        const data = Buffer.from(arrayBuffer);
        data.forEach((color, index) => {
          const y = Math.floor(index / height);
          const x = index % width;
          this.canvasRef.current.setCell(x, y, color);
        });
      })
      .catch(console.error);
  }

  connectToWebsocketAPI() {
    const ws = new WebSocket(`ws://${window.location.host}/api/connect`);
    ws.binaryType = 'arraybuffer';

    ws.addEventListener('message', event => {
      const packet = Buffer.from(event.data);
      console.log('received packet', packetToString(packet));

      const packetType = packet.readUInt32BE(0);
      switch (packetType) {
        case 0: {
          const errorLength = packet.readUInt32LE(4);
          const error = packet.toString('utf8', 8, errorLength);
          console.error(error);
          break;
        }

        case 1:
        case 2: {
          const x = packet.readUInt16BE(4);
          const y = packet.readUInt16BE(6);
          const color = packet.readUInt8(8);
          this.canvasRef.current.setCell(x, y, color);
          break;
        }

        default:
          break;
      }
    });

    return ws;
  }

  sendSetCell(x, y, selectedColor) {
    const packet = Buffer.alloc(9);
    packet.writeUInt32BE(1, 0);
    packet.writeUInt16BE(x, 4);
    packet.writeUInt16BE(y, 6);
    packet.writeUInt8(selectedColor, 8);
    this.ws.send(packet);
    console.log('sent packet', packetToString(packet));
  }

  render() {
    const { selectedColor } = this.state;

    return (
      <>
        <PanZoom onClick={this.onCanvasClick}>
          <CanvasRenderer ref={this.canvasRef} />
        </PanZoom>
        <ColorPicker value={selectedColor} onChange={this.onColorSelected} />
      </>
    );
  }
}

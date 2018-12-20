import React from 'react';
import { width, height, palette } from './config';
import CanvasRenderer from './CanvasRenderer';
import PanZoom from './PanZoom';
import ColorPicker from './ColorPicker';
import './App.scss';

export default class App extends React.Component {
  state = { selectedColor: Math.floor(Math.random() * palette.length) };

  canvasRef = React.createRef();

  componentDidMount() {
    this.fetchEntireCanvas();
  }

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

  render() {
    const { selectedColor } = this.state;

    return (
      <>
        <PanZoom>
          <CanvasRenderer ref={this.canvasRef} />
        </PanZoom>
        <ColorPicker value={selectedColor} onChange={this.onColorSelected} />
      </>
    );
  }
}

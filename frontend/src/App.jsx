import React from 'react';
import { width, height } from './config';
import CanvasRenderer from './CanvasRenderer';
import './App.scss';

export default class App extends React.Component {
  canvasRef = React.createRef();

  componentDidMount() {
    this.fetchEntireCanvas();
  }

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
    return <CanvasRenderer ref={this.canvasRef} />;
  }
}

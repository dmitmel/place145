import React from 'react';
import { width, height, palette } from './config';
import './CanvasRenderer.scss';

export default class CanvasRenderer extends React.Component {
  canvasRef = React.createRef();

  imageDataChanged = false;

  componentDidMount() {
    this.ctx = this.canvasRef.current.getContext('2d');
    this.imageData = this.ctx.createImageData(width, height);

    this.animationFrame = window.requestAnimationFrame(this.draw);
  }

  componentWillUnmount() {
    window.cancelAnimationFrame(this.animationFrame);
  }

  onTransform = transform => {
    this.transform = transform;
  };

  setCell(x, y, color) {
    const i = (y * width + x) * 4;
    const [r, g, b] = palette[color % palette.length];

    const { data } = this.imageData;
    data[i + 0] = r;
    data[i + 1] = g;
    data[i + 2] = b;
    data[i + 3] = 255;

    this.imageDataChanged = true;
  }

  draw = () => {
    if (this.imageDataChanged) {
      this.ctx.clearRect(0, 0, width, height);
      this.ctx.putImageData(this.imageData, 0, 0);
      this.imageDataChanged = false;
    }

    this.animationFrame = window.requestAnimationFrame(this.draw);
  };

  render() {
    return (
      <canvas
        className="CanvasRenderer canvas"
        width={width}
        height={height}
        ref={this.canvasRef}
        onClick={this.onClick}
      />
    );
  }
}

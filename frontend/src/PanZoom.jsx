/* eslint-disable lines-between-class-members */
/* eslint-disable jsx-a11y/click-events-have-key-events */
/* eslint-disable jsx-a11y/no-static-element-interactions */

import React from 'react';
import './PanZoom.scss';

const getMouseCoords = event => ({ x: event.clientX, y: event.clientY });

export default class PanZoom extends React.Component {
  state = { x: 0, y: 0 };
  stateBeforePan = this.state;
  mouseBeforePan = { x: 0, y: 0 };

  containerRef = React.createRef();
  childRef = React.createRef();

  onDragStart = ({ nativeEvent: event }) => {
    this.stateBeforePan = this.state;
    this.mouseBeforePan = getMouseCoords(event);

    document.addEventListener('mousemove', this.onDragMove);
    document.addEventListener('mouseup', this.onDragStop);
  };

  onDragMove = event => {
    const mouse = getMouseCoords(event);

    this.setState({
      x: this.stateBeforePan.x + (mouse.x - this.mouseBeforePan.x),
      y: this.stateBeforePan.y + (mouse.y - this.mouseBeforePan.y),
    });
  };

  onDragStop = () => {
    document.removeEventListener('mousemove', this.onDragMove);
    document.removeEventListener('mouseup', this.onDragStop);
  };

  render() {
    const { children } = this.props;
    const { x, y } = this.state;

    return (
      <div
        className="PanZoom container"
        ref={this.containerRef}
        onMouseDown={this.onDragStart}>
        <div
          style={{
            transform: `translate(${x}px,${y}px)`,
          }}
          ref={this.childRef}
          onClick={this.onClick}>
          {children}
        </div>
      </div>
    );
  }
}

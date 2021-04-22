import React from 'react';
import PropTypes from 'prop-types';
import './PanZoom.scss';

const SCROLL_ZOOM_SENSITIVITY = 0.01;
const CLICK_THRESHOLD = 4;

const getMouseCoords = event => ({ x: event.clientX, y: event.clientY });

export default class PanZoom extends React.Component {
  static propTypes = {
    onClick: PropTypes.func.isRequired,
  };

  state = { x: 0, y: 0, scale: 1 };
  stateBeforePan = this.state;
  mouseBeforePan = { x: 0, y: 0 };

  containerRef = React.createRef();
  childRef = React.createRef();

  componentDidMount() {
    this.setState(({ scale }) => {
      const container = this.containerRef.current;
      return {
        x: container.clientWidth / 2,
        y: container.clientHeight / 2,
        scale: this.adjustScale(scale),
      };
    });
  }

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

  onZoom = ({ nativeEvent: event }) => {
    this.setState(state => {
      const newScale = this.adjustScale(
        state.scale * (1 - event.deltaY * SCROLL_ZOOM_SENSITIVITY),
      );

      const zoomFactor = newScale / state.scale;
      const mouse = getMouseCoords(event);

      return {
        x: mouse.x - zoomFactor * (mouse.x - state.x),
        y: mouse.y - zoomFactor * (mouse.y - state.y),
        scale: newScale,
      };
    });
  };

  onClick = ({ nativeEvent: event }) => {
    const { state } = this;
    const { onClick } = this.props;

    const mouse = getMouseCoords(event);

    const child = this.childRef.current;
    const topLeftCorner = {
      x: state.x - (state.scale * child.clientWidth) / 2,
      y: state.y - (state.scale * child.clientHeight) / 2,
    };

    const x = Math.floor((mouse.x - topLeftCorner.x) / state.scale);
    const y = Math.floor((mouse.y - topLeftCorner.y) / state.scale);

    const dx = state.x - this.stateBeforePan.x;
    const dy = state.y - this.stateBeforePan.y;
    if (dx * dx + dy * dy <= CLICK_THRESHOLD * CLICK_THRESHOLD) onClick(x, y);
  };

  adjustScale(scale) {
    const container = this.containerRef.current;
    const child = this.childRef.current;

    const isLandscape = container.clientWidth > container.clientHeight;

    const maxScale =
      (isLandscape ? container.clientHeight : container.clientWidth) / 2;
    const minScale =
      maxScale / (isLandscape ? child.clientHeight : child.clientWidth);

    return Math.max(minScale, Math.min(scale, maxScale));
  }

  render() {
    const { children } = this.props;
    const { x, y, scale } = this.state;

    return (
      <div
        className="PanZoom container"
        ref={this.containerRef}
        onMouseDown={this.onDragStart}
        onWheel={this.onZoom}>
        <div
          style={{
            transform: `translate(-50%,-50%) translate(${x}px,${y}px) scale(${scale})`,
          }}
          ref={this.childRef}
          onClick={this.onClick}>
          {children}
        </div>
      </div>
    );
  }
}

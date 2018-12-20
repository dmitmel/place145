import React from 'react';
import PropTypes from 'prop-types';
import { palette } from './config';
import byteToString from './byteToString';
import './ColorPicker.scss';

ColorPicker.propTypes = {
  value: PropTypes.number.isRequired,
  onChange: PropTypes.func.isRequired,
};

export default function ColorPicker({ value, onChange }) {
  return (
    <div id="colors-wrapper">
      <div id="colors">
        {palette.map((color, index) => (
          <Color
            key={color}
            value={color}
            selected={index === value}
            onSelect={() => onChange(index)}
          />
        ))}
      </div>
    </div>
  );
}

Color.propTypes = {
  value: PropTypes.arrayOf(PropTypes.number.isRequired),
  selected: PropTypes.bool.isRequired,
  onSelect: PropTypes.func.isRequired,
};

function Color({ value, selected, onSelect }) {
  const hex = value.map(byteToString).join('');
  const id = `color${hex}`;

  return (
    <label
      key={value}
      htmlFor={id}
      className={selected ? 'selected' : null}
      style={{ backgroundColor: `#${hex}` }}>
      <input
        type="radio"
        name="color"
        id={id}
        checked={selected}
        onChange={onSelect}
      />
    </label>
  );
}

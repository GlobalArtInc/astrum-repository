

import React from 'react';
import { render, } from '@testing-library/react';
import { NodeIcon } from '../NodeIcon';

describe('NodeIcon', () => {
  describe('renders correct icon for each type', () => {
    it('renders folder icon for folder type', () => {
      const { container } = render(<NodeIcon type="folder" />);
      const svg = container.querySelector('svg');
      expect(svg).toBeInTheDocument();
      // Lucide icons have class names indicating the icon type
      expect(svg).toHaveClass('lucide-folder');
    });

    it('renders package icon for component type', () => {
      const { container } = render(<NodeIcon type="component" />);
      const svg = container.querySelector('svg');
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveClass('lucide-package');
    });

    it('renders file icon for asset type', () => {
      const { container } = render(<NodeIcon type="asset" />);
      const svg = container.querySelector('svg');
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveClass('lucide-file');
    });
  });

  describe('size prop', () => {
    it('uses default size of 16', () => {
      const { container } = render(<NodeIcon type="folder" />);
      const svg = container.querySelector('svg');
      expect(svg).toHaveAttribute('width', '16');
      expect(svg).toHaveAttribute('height', '16');
    });

    it('uses custom size when provided', () => {
      const { container } = render(<NodeIcon type="folder" size={24} />);
      const svg = container.querySelector('svg');
      expect(svg).toHaveAttribute('width', '24');
      expect(svg).toHaveAttribute('height', '24');
    });
  });

  describe('className prop', () => {
    it('applies custom className', () => {
      const { container } = render(<NodeIcon type="folder" className="custom-class" />);
      const svg = container.querySelector('svg');
      expect(svg).toHaveClass('custom-class');
    });
  });
});


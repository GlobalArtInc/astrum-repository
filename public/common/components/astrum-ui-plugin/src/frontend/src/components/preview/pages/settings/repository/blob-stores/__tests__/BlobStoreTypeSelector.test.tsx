

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';
import { BlobStoreTypeSelector } from '../BlobStoreTypeSelector';
import { BLOB_STORE_TYPE_IDS } from '../blobStoreFormMachine';

const renderWithTheme = (ui: React.ReactElement) =>
  render(<Theme>{ui}</Theme>);

describe('BlobStoreTypeSelector', () => {
  it('renders all blob store type options', () => {
    renderWithTheme(
      <BlobStoreTypeSelector selectedType={null} onSelect={jest.fn()} />
    );

    expect(screen.getByText('File')).toBeInTheDocument();
    expect(screen.getByText('Amazon S3')).toBeInTheDocument();
    expect(screen.getByText('Azure Blob')).toBeInTheDocument();
    expect(screen.getByText('Google Cloud')).toBeInTheDocument();
    expect(screen.getByText('Group')).toBeInTheDocument();
  });

  it('calls onSelect when a type is clicked', () => {
    const onSelect = jest.fn();
    renderWithTheme(
      <BlobStoreTypeSelector selectedType={null} onSelect={onSelect} />
    );

    fireEvent.click(screen.getByText('File'));
    expect(onSelect).toHaveBeenCalledWith(BLOB_STORE_TYPE_IDS.FILE);

    fireEvent.click(screen.getByText('Amazon S3'));
    expect(onSelect).toHaveBeenCalledWith(BLOB_STORE_TYPE_IDS.S3);
  });

  it('shows selected state for chosen type', () => {
    renderWithTheme(
      <BlobStoreTypeSelector
        selectedType={BLOB_STORE_TYPE_IDS.FILE}
        onSelect={jest.fn()}
      />
    );

    const fileCard = screen.getByText('File').closest('button');
    expect(fileCard).toHaveAttribute('aria-selected', 'true');
  });

  it('does not call onSelect when disabled', () => {
    const onSelect = jest.fn();
    renderWithTheme(
      <BlobStoreTypeSelector selectedType={null} onSelect={onSelect} disabled />
    );

    fireEvent.click(screen.getByText('File'));
    expect(onSelect).not.toHaveBeenCalled();
  });
});

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { CargoFacet } from '../CargoFacet';
import { ConanFacet } from '../ConanFacet';
import { TerraformFacet } from '../TerraformFacet';
import { RepositoryFormData } from '../../types';

const baseFormData: RepositoryFormData = {
  name: 'test',
  format: 'cargo',
  type: 'proxy',
  recipe: 'cargo-proxy',
  online: true,
  storage: { blobStoreName: 'default', strictContentTypeValidation: true },
};

function renderInTheme(ui: React.ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

describe('CargoFacet', () => {
  it('shows the authentication requirement and turns it on', () => {
    const onNestedChange = jest.fn();
    renderInTheme(<CargoFacet formData={baseFormData} onNestedChange={onNestedChange} />);

    expect(screen.getByText('Cargo Settings')).toBeInTheDocument();
    const checkbox = screen.getByRole('checkbox', { name: /Restrict repository content to authenticated users/ });
    expect(checkbox).not.toBeChecked();

    fireEvent.click(checkbox);

    expect(onNestedChange).toHaveBeenCalledWith('cargo', { requireAuthentication: true });
  });

  it('shows a saved requirement as checked', () => {
    renderInTheme(
      <CargoFacet formData={{ ...baseFormData, cargo: { requireAuthentication: true } }} onNestedChange={jest.fn()} />
    );

    expect(screen.getByRole('checkbox', { name: /Restrict repository content/ })).toBeChecked();
  });
});

describe('ConanFacet', () => {
  const conanData = { ...baseFormData, format: 'conan', recipe: 'conan-proxy' };

  it('offers the protocol version when the proxy is created', () => {
    renderInTheme(<ConanFacet formData={{ ...conanData, conanProxy: { conanVersion: 'V2' } }} onNestedChange={jest.fn()} isEdit={false} />);

    expect(screen.getByText('Protocol version')).toBeInTheDocument();
    expect(screen.getByRole('combobox')).toHaveTextContent('Conan V2');
  });

  it('shows the saved protocol version read-only when the proxy is edited', () => {
    renderInTheme(<ConanFacet formData={{ ...conanData, conanProxy: { conanVersion: 'V1' } }} onNestedChange={jest.fn()} isEdit />);

    const field = screen.getByDisplayValue('Conan V1');
    expect(field).toHaveAttribute('readonly');
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });
});

describe('TerraformFacet', () => {
  const terraformData = { ...baseFormData, format: 'terraform', type: 'hosted' as const, recipe: 'terraform-hosted' };

  it('shows the signing key and an empty passphrase on edit', () => {
    renderInTheme(
      <TerraformFacet formData={{ ...terraformData, terraformSigning: { keypair: 'KEY' } }} onNestedChange={jest.fn()} isEdit />
    );

    expect(screen.getByText('Terraform Settings')).toBeInTheDocument();
    expect(screen.getByDisplayValue('KEY')).toBeInTheDocument();
    const passphrase = document.querySelector('input[name="terraform-passphrase"]') as HTMLInputElement;
    expect(passphrase).toHaveAttribute('type', 'password');
    expect(passphrase.value).toBe('');
    expect(screen.getByText(/The saved passphrase is never shown/)).toBeInTheDocument();
  });

  it('passes typed values on', () => {
    const onNestedChange = jest.fn();
    renderInTheme(<TerraformFacet formData={terraformData} onNestedChange={onNestedChange} isEdit={false} />);

    fireEvent.change(document.querySelector('input[name="terraform-passphrase"]') as HTMLInputElement, { target: { value: 'pw' } });

    expect(onNestedChange).toHaveBeenCalledWith('terraformSigning', { passphrase: 'pw' });
  });

  it('shows a missing key error', () => {
    renderInTheme(
      <TerraformFacet
        formData={terraformData}
        onNestedChange={jest.fn()}
        errors={{ 'terraformSigning.keypair': 'A signing key is required when a passphrase is set' }}
        isEdit={false}
      />
    );

    expect(screen.getByText('A signing key is required when a passphrase is set')).toBeInTheDocument();
  });
});

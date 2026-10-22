import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Providers, testClient } from '../../../test/render';
import { FilePreview, type FilePreviewProps } from './FilePreview';

const URL_OF = (file: string) =>
  `/service/rest/internal/ui/browse/raw/content?path=${encodeURIComponent(file).replace(/%20/g, '+')}`;

function serve(files: Record<string, BodyInit>) {
  const fetch = vi.fn(async (input: RequestInfo | URL) => {
    const body = files[String(input)];
    return body === undefined
      ? new Response(JSON.stringify({ message: 'missing' }), { status: 404 })
      : new Response(body, { status: 200 });
  });
  vi.stubGlobal('fetch', fetch);
  return fetch;
}

function show(props: Partial<FilePreviewProps> & { file: string }) {
  return render(
    <Providers client={testClient()}>
      <FilePreview
        repository="raw"
        name={props.file.split('/').pop() ?? props.file}
        contentType="text/plain"
        size={100}
        {...props}
      />
    </Providers>,
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the file preview', () => {
  it('shows text with line numbers', async () => {
    const fetch = serve({ [URL_OF('/install.sh')]: '#!/bin/sh\necho hello\n' });
    show({ file: '/install.sh' });
    expect(await screen.findByText(/echo hello/)).toBeInTheDocument();
    expect(
      screen.getByText((_, element) => element?.tagName === 'PRE' && element.textContent === '1\n2'),
    ).toHaveAttribute('aria-hidden', 'true');
    expect(screen.getByRole('button', { name: 'Copy the content' })).toBeInTheDocument();
    expect(fetch).toHaveBeenCalledWith(URL_OF('/install.sh'), expect.objectContaining({ method: 'GET' }));
  });

  it('renders Markdown and shows its source on request', async () => {
    serve({ [URL_OF('/0.2.2/INSTALL.md')]: '# Install\n\nRun `install.sh`.\n' });
    show({ file: '/0.2.2/INSTALL.md', contentType: 'text/markdown' });
    expect(await screen.findByRole('heading', { name: 'Install' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('radio', { name: 'Source' }));
    expect(screen.queryByRole('heading', { name: 'Install' })).not.toBeInTheDocument();
    expect(screen.getByText(/# Install/)).toBeInTheDocument();
  });

  it('explains binary, empty and large files', async () => {
    serve({
      [URL_OF('/blob.bin')]: new Uint8Array([0x00, 0x01, 0xff]),
      [URL_OF('/empty.txt')]: '',
    });
    const binary = show({ file: '/blob.bin', contentType: 'application/octet-stream' });
    expect(await screen.findByText(/This file is binary/)).toBeInTheDocument();
    binary.unmount();
    const empty = show({ file: '/empty.txt' });
    expect(await screen.findByText('The file is empty.')).toBeInTheDocument();
    empty.unmount();
    show({ file: '/big.log', size: 5 * 1024 * 1024 });
    expect(screen.getByText(/too large to preview/)).toBeInTheDocument();
  });

  it('shows images and skips archives', () => {
    const image = show({ file: '/logo.png', contentType: 'image/png' });
    expect(screen.getByRole('region', { name: 'Preview' })).toBeInTheDocument();
    expect(screen.getByRole('img', { name: 'logo.png' })).toHaveAttribute('src', URL_OF('/logo.png'));
    image.unmount();
    show({ file: '/app.jar', contentType: 'application/java-archive' });
    expect(screen.queryByRole('region', { name: 'Preview' })).not.toBeInTheDocument();
  });
});

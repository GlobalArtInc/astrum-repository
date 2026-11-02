import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Providers, testClient } from '../../../test/render';
import type { AssetDetails } from '../types';
import { AssetSummary } from './AssetSummary';

const HEX = '0123456789abcdef'.repeat(4);

function details(extra: Partial<AssetDetails> = {}): AssetDetails {
  return {
    id: 'asset-1',
    path: '/v1/tag',
    repository: 'images',
    browsedRepository: 'images',
    format: 'docker',
    downloadUrl: 'http://localhost/repository/images/v1/tag',
    checksum: { sha256: HEX },
    contentType: 'application/vnd.docker.distribution.manifest.v2+json',
    lastModified: '2026-10-01T10:00:00.000+00:00',
    lastDownloaded: null,
    uploader: 'admin',
    uploaderIp: '127.0.0.1',
    fileSize: 1024,
    blobCreated: '2026-10-01T10:00:00.000+00:00',
    blobUpdated: '2026-10-02T12:00:00.000+00:00',
    blobStoreName: 'default',
    lastVerified: null,
    downloadCount: 7,
    component: null,
    canDelete: true,
    image: { size: 2048, layers: 3 },
    ...extra,
  };
}

function summary(asset: AssetDetails) {
  render(
    <Providers client={testClient()}>
      <AssetSummary details={asset} />
    </Providers>,
  );
  return screen.getByRole('region', { name: 'Summary' });
}

describe('the asset summary', () => {
  it('shows the push time, the size with layers, the shortened digest and the pulls of a docker tag', () => {
    const region = summary(details());
    expect(within(region).getByText('Pushed')).toBeInTheDocument();
    expect(within(region).getByText(/3 layers/)).toBeInTheDocument();
    expect(within(region).getByTitle(`sha256:${HEX}`)).toHaveTextContent('0123456789…cdef');
    expect(within(region).getByRole('button', { name: 'Copy the digest' })).toBeInTheDocument();
    expect(within(region).getByText('Pulls').nextElementSibling).toHaveTextContent('7');
  });

  it('shows the upload time, the size and the pulls of a file without a digest tile', () => {
    const region = summary(details({ format: 'raw', image: undefined }));
    expect(within(region).getByText('Uploaded')).toBeInTheDocument();
    expect(within(region).queryByText('Pushed')).not.toBeInTheDocument();
    expect(within(region).getByText('Size')).toBeInTheDocument();
    expect(within(region).queryByText(/layers/)).not.toBeInTheDocument();
    expect(within(region).getByText('Pulls')).toBeInTheDocument();
    expect(within(region).queryByText('Digest')).not.toBeInTheDocument();
  });

  it('shows no digest tile for a docker tag without a sha256 checksum', () => {
    const region = summary(details({ checksum: { sha1: 'a'.repeat(40) } }));
    expect(within(region).getByText('Pushed')).toBeInTheDocument();
    expect(within(region).queryByText('Digest')).not.toBeInTheDocument();
    expect(within(region).queryByRole('button', { name: 'Copy the digest' })).not.toBeInTheDocument();
  });
});

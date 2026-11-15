import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { SbomPage, SupplyChain } from './api';
import { sbomUrl, supplyChainUrl } from './model';
import { ImageSupplyChain, SigningChip } from './SupplyChain';

type Reveal = (entries: IntersectionObserverEntry[]) => void;

const DIGEST = `sha256:${'ab'.repeat(32)}`;
const SBOM = `sha256:${'cd'.repeat(32)}`;
const SUPPLY = supplyChainUrl('images', 'app', '1.0');
const FIRST_PAGE = sbomUrl('images', 'app', SBOM, 1, '');

function chain(overrides: Partial<SupplyChain> = {}): SupplyChain {
  return { digest: DIGEST, children: [], signatures: [], attestations: [], sboms: [], ...overrides };
}

function sbomPage(total: number, names: string[]): SbomPage {
  return {
    format: 'spdx',
    total,
    page: 1,
    pageSize: 100,
    packages: names.map((name) => ({
      name,
      version: '1.0.0',
      purl: `pkg:npm/${name}@1.0.0`,
      licenses: ['MIT'],
      ecosystem: 'npm',
    })),
  };
}

function serve(handlers: Record<string, () => Response>) {
  const fake = fakeServer(handlers);
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

function renderImage() {
  return renderWithApp(<ImageSupplyChain repository="images" image="app" reference="1.0" />);
}

function withSbom(extra: Record<string, () => Response>) {
  return serve({
    [`GET ${SUPPLY}`]: () => jsonResponse(chain({ sboms: [{ digest: SBOM, format: 'spdx' }] })),
    ...extra,
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the signatures panel', () => {
  it('shows a verified signature with its signer and scheme', async () => {
    serve({
      [`GET ${SUPPLY}`]: () =>
        jsonResponse(
          chain({
            signatures: [
              {
                digest: `sha256:${'ef'.repeat(32)}`,
                scheme: 'bundle',
                state: 'verified',
                signer: 'release-key',
              },
            ],
          }),
        ),
    });
    renderImage();
    expect(await screen.findByText('Verified by release-key')).toBeInTheDocument();
    expect(screen.getByText('Bundle')).toBeInTheDocument();
    expect(screen.getByTitle(`sha256:${'ef'.repeat(32)}`)).toHaveTextContent('sha256:efefefefefef…');
  });

  it('shows invalid and unsupported signatures, and an unknown state when the server omits it', async () => {
    serve({
      [`GET ${SUPPLY}`]: () =>
        jsonResponse(
          chain({
            signatures: [
              { digest: `sha256:${'01'.repeat(32)}`, scheme: 'legacy', state: 'invalid' },
              { digest: `sha256:${'02'.repeat(32)}`, scheme: 'notation', state: 'unsupported' },
              { digest: `sha256:${'03'.repeat(32)}`, scheme: 'referrer' },
            ],
          }),
        ),
    });
    renderImage();
    expect(await screen.findByText('Invalid')).toBeInTheDocument();
    expect(screen.getByText('Unsupported')).toBeInTheDocument();
    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('says that no signature is attached when the list is empty', async () => {
    serve({ [`GET ${SUPPLY}`]: () => jsonResponse(chain()) });
    renderImage();
    expect(await screen.findByText('No signatures are attached to this image')).toBeInTheDocument();
  });

  it('counts the attestations and lists them apart from the signatures', async () => {
    serve({
      [`GET ${SUPPLY}`]: () =>
        jsonResponse(
          chain({
            attestations: [{ digest: `sha256:${'11'.repeat(32)}` }, { digest: `sha256:${'22'.repeat(32)}` }],
          }),
        ),
    });
    renderImage();
    expect(await screen.findByText('2 attestations')).toBeInTheDocument();
    expect(screen.getByText('No signatures are attached to this image')).toBeInTheDocument();
  });

  it('loads the supply chain only once the panels come into view', async () => {
    let reveal: Reveal = () => undefined;
    class FakeObserver {
      constructor(callback: Reveal) {
        reveal = callback;
      }
      observe() {}
      disconnect() {}
    }
    vi.stubGlobal('IntersectionObserver', FakeObserver);
    const fake = serve({ [`GET ${SUPPLY}`]: () => jsonResponse(chain()) });
    renderImage();
    expect(fake.calls).toHaveLength(0);
    act(() => reveal([{ isIntersecting: true } as IntersectionObserverEntry]));
    expect(await screen.findByText('No signatures are attached to this image')).toBeInTheDocument();
    expect(fake.calls.map((call) => call.path)).toEqual([SUPPLY]);
  });
});

describe('the SBOM panel', () => {
  it('lists the first page of packages with the total and the pages', async () => {
    withSbom({ [`GET ${FIRST_PAGE}`]: () => jsonResponse(sbomPage(150, ['left-pad', 'lodash'])) });
    renderImage();
    expect(await screen.findByText('left-pad')).toBeInTheDocument();
    expect(screen.getByText('lodash')).toBeInTheDocument();
    expect(screen.getByText('150 packages')).toBeInTheDocument();
    expect(screen.getByText('Page 1 of 2')).toBeInTheDocument();
  });

  it('moves to the next page', async () => {
    const second = sbomUrl('images', 'app', SBOM, 2, '');
    const fake = withSbom({
      [`GET ${FIRST_PAGE}`]: () => jsonResponse(sbomPage(150, ['left-pad'])),
      [`GET ${second}`]: () => jsonResponse(sbomPage(150, ['zlib'])),
    });
    renderImage();
    await screen.findByText('left-pad');
    await userEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(await screen.findByText('zlib')).toBeInTheDocument();
    expect(fake.calls.map((call) => call.path)).toContain(second);
  });

  it('goes back to the first page when the filter changes', async () => {
    const second = sbomUrl('images', 'app', SBOM, 2, '');
    const filtered = sbomUrl('images', 'app', SBOM, 1, 'log');
    const fake = withSbom({
      [`GET ${FIRST_PAGE}`]: () => jsonResponse(sbomPage(150, ['left-pad'])),
      [`GET ${second}`]: () => jsonResponse(sbomPage(150, ['zlib'])),
      [`GET ${filtered}`]: () => jsonResponse(sbomPage(1, ['log4js'])),
    });
    renderImage();
    await screen.findByText('left-pad');
    await userEvent.click(screen.getByRole('button', { name: 'Next' }));
    await screen.findByText('zlib');
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter packages' }), 'log');
    expect(await screen.findByText('log4js')).toBeInTheDocument();
    expect(fake.calls.map((call) => call.path)).toContain(filtered);
    expect(screen.getByText('1 package')).toBeInTheDocument();
  });

  it('says that the SBOM cannot be read on 422', async () => {
    withSbom({ [`GET ${FIRST_PAGE}`]: () => jsonResponse({ id: '*', message: 'unreadable' }, 422) });
    renderImage();
    expect(await screen.findByText('This SBOM cannot be read')).toBeInTheDocument();
  });

  it('says that the SBOM is too large on 413', async () => {
    withSbom({ [`GET ${FIRST_PAGE}`]: () => jsonResponse({ id: '*', message: 'large' }, 413) });
    renderImage();
    expect(await screen.findByText('This SBOM is too large to show')).toBeInTheDocument();
  });

  it('shows the wait from Retry-After on 503 and keeps Retry until it passes', async () => {
    withSbom({
      [`GET ${FIRST_PAGE}`]: () =>
        new Response(JSON.stringify({ id: '*', message: 'busy' }), {
          status: 503,
          headers: { 'content-type': 'application/json', 'Retry-After': '5' },
        }),
    });
    renderImage();
    expect(
      await screen.findByText('The server is busy reading SBOMs. Try again in 5 s.'),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Retry' })).toBeDisabled();
  });

  it('says that the SBOM lists no packages when it is empty', async () => {
    withSbom({ [`GET ${FIRST_PAGE}`]: () => jsonResponse(sbomPage(0, [])) });
    renderImage();
    expect(await screen.findByText('This SBOM lists no packages')).toBeInTheDocument();
  });
});

describe('the signing chip on a tag', () => {
  it('shows the signer of a verified signature', async () => {
    serve({
      [`GET ${SUPPLY}`]: () =>
        jsonResponse(
          chain({
            signatures: [{ digest: DIGEST, scheme: 'bundle', state: 'verified', signer: 'release-key' }],
          }),
        ),
    });
    renderWithApp(<SigningChip repository="images" image="app" reference="1.0" />);
    expect(await screen.findByText('Signed by release-key')).toBeInTheDocument();
  });

  it('says that the tag is not signed when no signature is verified', async () => {
    serve({ [`GET ${SUPPLY}`]: () => jsonResponse(chain({ signatures: [] })) });
    renderWithApp(<SigningChip repository="images" image="app" reference="1.0" />);
    expect(await screen.findByText('Not signed')).toBeInTheDocument();
  });
});

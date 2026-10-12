import { interpret } from 'xstate';
import { waitFor } from 'xstate/lib/waitFor';
import { createRepositoryFormMachine, validateRepository } from '../repositoryFormMachine';
import { buildRepositoryConfig } from '../useRepositoriesApi';
import { RepositoryFormData } from '../types';

jest.mock('../../../../../../../interface/api', () => ({
  ...jest.requireActual('../../../../../../../interface/api'),
  API_INTERNAL_UI: '/service/rest/internal/ui',
  ENDPOINTS: {
    REPOSITORIES: '/service/rest/v1/repositories',
    BLOBSTORES: '/service/rest/v1/blobstores',
    ROUTING_RULES: '/service/rest/v1/routing-rules',
  },
  restClient: {
    get: jest.fn().mockResolvedValue([]),
  },
}));

const { restClient } = jest.requireMock('../../../../../../../interface/api');

async function loadedData(options: Parameters<typeof createRepositoryFormMachine>[0]) {
  restClient.get.mockImplementation((url: string) =>
    Promise.resolve(url.includes('/blobstores') ? [{ name: 'default', type: 'File' }] : []));
  const service = interpret(createRepositoryFormMachine(options)).start();
  await waitFor(service, (state) => state.matches('editing'));
  const data = service.getSnapshot().context.data as RepositoryFormData;
  service.stop();
  return data;
}

const base = {
  online: true,
  storage: { blobStoreName: 'default', strictContentTypeValidation: true },
};

function formData(format: string, type: 'hosted' | 'proxy' | 'group', extra: Partial<RepositoryFormData> = {}) {
  return {
    name: `${format}-${type}`,
    format,
    type,
    recipe: `${format}-${type}`,
    ...base,
    ...(type === 'proxy' ? { proxy: { remoteUrl: 'https://example.com', contentMaxAge: 1440, metadataMaxAge: 1440 } } : {}),
    ...(type === 'group' ? { group: { memberNames: ['a'] } } : {}),
    ...extra,
  } as RepositoryFormData;
}

describe('format attributes of the repository form', () => {
  describe('defaults for a new repository', () => {
    it.each(['hosted', 'proxy', 'group'] as const)('lets anonymous users read a new cargo %s repository', async (type) => {
      const data = await loadedData({ format: 'cargo', repositoryType: type });
      expect(data.cargo).toEqual({ requireAuthentication: false });
    });

    it('creates Conan proxies for Conan 2', async () => {
      const data = await loadedData({ format: 'conan', repositoryType: 'proxy' });
      expect(data.conanProxy).toEqual({ conanVersion: 'V2' });
    });

  });

  describe('editing a saved repository', () => {
    it('loads the cargo and conan attributes', async () => {
      const cargo = await loadedData({
        format: 'cargo',
        repositoryName: 'crates',
        preloadedRepository: { name: 'crates', type: 'proxy', format: 'cargo', cargo: { requireAuthentication: true } } as any,
      });
      expect(cargo.cargo).toEqual({ requireAuthentication: true });

      const conan = await loadedData({
        format: 'conan',
        repositoryName: 'conan-v1',
        preloadedRepository: { name: 'conan-v1', type: 'proxy', format: 'conan', conanProxy: { conanVersion: 'V1' } } as any,
      });
      expect(conan.conanProxy).toEqual({ conanVersion: 'V1' });
    });

    it('never puts the saved Terraform passphrase into the form', async () => {
      const data = await loadedData({
        format: 'terraform',
        repositoryName: 'tf',
        preloadedRepository: {
          name: 'tf', type: 'hosted', format: 'terraform', terraformSigning: { keypair: 'KEY', passphrase: 'secret' },
        } as any,
      });
      expect(data.terraformSigning).toEqual({ keypair: 'KEY' });
    });

    it('loads the Terraform settings', async () => {
      const data = await loadedData({
        format: 'terraform',
        repositoryName: 'tf-group',
        preloadedRepository: {
          name: 'tf-group', type: 'group', format: 'terraform',
          terraform: { requireAuthentication: true, registryHost: 'terraform.example.com' },
        } as any,
      });
      expect(data.terraform).toEqual({ requireAuthentication: true, registryHost: 'terraform.example.com' });
    });
  });

  describe('validation', () => {
    it('asks for a Terraform signing key when only a passphrase is given', () => {
      const errors = validateRepository(formData('terraform', 'hosted', { terraformSigning: { keypair: '', passphrase: 'pw' } }));
      expect(errors['terraformSigning.keypair']).toBe('A signing key is required when a passphrase is set');
    });

    it('accepts a Terraform hosted repository without signing', () => {
      expect(validateRepository(formData('terraform', 'hosted'))).toEqual({});
    });

    it('checks the Terraform host names', () => {
      const message = 'Enter a host name, optionally followed by :port, without https:// or a path';
      const errors = validateRepository(formData('terraform', 'proxy', {
        terraform: { registryHost: 'https://terraform.example.com/', upstreamHost: 'registry terraform' },
      }));
      expect(errors['terraform.registryHost']).toBe(message);
      expect(errors['terraform.upstreamHost']).toBe(message);
      expect(validateRepository(formData('terraform', 'group', {
        terraform: { registryHost: 'terraform.example.com:8443' },
      }))).toEqual({});
    });
  });

  describe('REST payload', () => {
    it.each(['hosted', 'proxy', 'group'] as const)('sends cargo.requireAuthentication for cargo %s', (type) => {
      expect(buildRepositoryConfig(formData('cargo', type, { cargo: { requireAuthentication: true } })).cargo)
        .toEqual({ requireAuthentication: true });
      expect(buildRepositoryConfig(formData('cargo', type)).cargo).toEqual({ requireAuthentication: false });
    });

    it('sends no cargo block for other formats', () => {
      expect(buildRepositoryConfig(formData('raw', 'hosted')).cargo).toBeUndefined();
    });

    it('always sends the Conan version of a proxy so an edit cannot turn Conan 1 into Conan 2', () => {
      expect(buildRepositoryConfig(formData('conan', 'proxy', { conanProxy: { conanVersion: 'V1' } })).conanProxy)
        .toEqual({ conanVersion: 'V1' });
      expect(buildRepositoryConfig(formData('conan', 'proxy')).conanProxy).toEqual({ conanVersion: 'V2' });
      expect(buildRepositoryConfig(formData('conan', 'hosted')).conanProxy).toBeUndefined();
    });

    it('sends the Terraform settings, with the upstream host for proxies only', () => {
      expect(buildRepositoryConfig(formData('terraform', 'proxy', {
        terraform: { requireAuthentication: true, registryHost: ' tf.example.com ', upstreamHost: 'registry.terraform.io' },
      })).terraform).toEqual({ requireAuthentication: true, registryHost: 'tf.example.com', upstreamHost: 'registry.terraform.io' });
      expect(buildRepositoryConfig(formData('terraform', 'group', { terraform: { upstreamHost: 'x.example.com' } })).terraform)
        .toEqual({ requireAuthentication: false, registryHost: null });
      expect(buildRepositoryConfig(formData('raw', 'hosted')).terraform).toBeUndefined();
    });

    it('sends the Terraform passphrase only when one was typed', () => {
      expect(buildRepositoryConfig(formData('terraform', 'hosted', { terraformSigning: { keypair: ' KEY ', passphrase: 'p w' } })).terraformSigning)
        .toEqual({ keypair: 'KEY', passphrase: 'p w' });
      expect(buildRepositoryConfig(formData('terraform', 'hosted', { terraformSigning: { keypair: 'KEY' } })).terraformSigning)
        .toEqual({ keypair: 'KEY' });
      expect(buildRepositoryConfig(formData('terraform', 'hosted', { terraformSigning: { keypair: 'KEY', passphrase: '' } })).terraformSigning)
        .toEqual({ keypair: 'KEY' });
      expect(buildRepositoryConfig(formData('terraform', 'hosted')).terraformSigning).toBeUndefined();
    });
  });
});

import { getFormatDescription, getRecipeDescription } from '../repositoryTypeDescriptions';

describe('repository type descriptions', () => {
  it('describes the Swift recipes and format', () => {
    expect(getRecipeDescription('swift', 'hosted')).toBe('Store your own Swift packages in a private package registry.');
    expect(getRecipeDescription('swift', 'proxy')).toBe('Cache Swift packages from another Swift package registry.');
    expect(getRecipeDescription('swift', 'group')).toBe('Combine multiple Swift package registries into a single URL.');
    expect(getFormatDescription('swift')).toBe('Package registry for the Swift Package Manager.');
  });

  it('falls back to a generic text for unknown recipes', () => {
    expect(getRecipeDescription('unknown', 'hosted')).toBe('Create a unknown (hosted) repository.');
  });
});

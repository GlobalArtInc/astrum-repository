import CapabilitiesStrings from '../../../../../constants/pages/settings/system/CapabilitiesStrings';
import { getCategoryForType, getTagLabel, getTagValue } from '../types';

describe('capability tags', () => {
  it('labels the known tags with UI strings', () => {
    expect(getTagLabel('Category')).toBe(CapabilitiesStrings.LIST.CATEGORY);
    expect(getTagLabel('Repository')).toBe(CapabilitiesStrings.LIST.REPOSITORY);
  });

  it('keeps the key of an unknown tag', () => {
    expect(getTagLabel('Custom')).toBe('Custom');
  });

  it('shows the category of the type instead of the server value', () => {
    expect(getTagValue({ typeId: 'scheduling.scheduler' }, 'Category', 'Scheduling')).toBe(
      CapabilitiesStrings.LIST.SCHEDULING
    );
    expect(getTagValue({ typeId: 'unknown.type' }, 'Category', 'Anything')).toBe(
      CapabilitiesStrings.LIST.CATEGORY_OTHER
    );
  });

  it('keeps the value of other tags', () => {
    expect(getTagValue({ typeId: 'webhook.repository' }, 'Repository', 'maven-releases')).toBe('maven-releases');
  });

  it('maps capability types to categories', () => {
    expect(getCategoryForType('audit')).toBe(CapabilitiesStrings.LIST.CATEGORY_AUDIT);
  });
});

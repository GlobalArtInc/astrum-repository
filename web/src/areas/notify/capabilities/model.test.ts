import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../../i18n';
import {
  aboutParagraphs,
  type CapabilityType,
  initialCapabilityValues,
  toCapabilityRequest,
  validateCapability,
} from './model';

const t = makeTranslate('en');

const field = (id: string, type: string, extra: Partial<CapabilityType['formFields'][number]> = {}) => ({
  id,
  type,
  label: id,
  helpText: null,
  required: true,
  initialValue: null,
  minValue: null,
  maxValue: null,
  ...extra,
});

const UI: CapabilityType = {
  id: 'rapture.settings',
  name: 'UI: Settings',
  about: 'Adjusts the user interface.',
  formFields: [
    field('title', 'string', { initialValue: 'GlobalArt Astrum Repository' }),
    field('debugAllowed', 'checkbox', { required: false, initialValue: 'true' }),
    field('requestTimeout', 'number', { initialValue: '60', minValue: '30' }),
    field('url', 'url', { required: false }),
  ],
};

describe('capability forms', () => {
  it('start from the stored values or the defaults of the type', () => {
    const values = initialCapabilityValues(UI, null);
    expect(values).toEqual({
      enabled: true,
      notes: '',
      properties: {
        title: 'GlobalArt Astrum Repository',
        debugAllowed: 'true',
        requestTimeout: '60',
        url: '',
      },
    });
    expect(
      toCapabilityRequest(UI, { ...values, properties: { ...values.properties, title: ' Team ' } }),
    ).toEqual({
      type: 'rapture.settings',
      enabled: true,
      notes: '',
      properties: { title: 'Team', debugAllowed: 'true', requestTimeout: '60', url: '' },
    });
  });

  it('check required fields, numbers and URLs', () => {
    const values = initialCapabilityValues(UI, null);
    expect(
      validateCapability(t, UI, {
        ...values,
        properties: { title: ' ', debugAllowed: 'false', requestTimeout: '10', url: 'ftp://x' },
      }),
    ).toEqual({
      title: 'Fill in “title”',
      requestTimeout: '“requestTimeout” must be at least 30',
      url: 'Enter a URL such as https://repo.example.com',
    });
  });

  it('turn the about text into paragraphs', () => {
    expect(aboutParagraphs('First line.\n\n<p>Second <em>part</em>.</p>')).toEqual([
      'First line.',
      'Second',
      'part',
      '.',
    ]);
  });
});

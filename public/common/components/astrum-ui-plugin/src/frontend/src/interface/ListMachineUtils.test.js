

import ListMachineUtils from './ListMachineUtils';

describe('ListMachineUtils', () => {
  const FIELD = 'field';

  describe('sortDataByFieldAndDirection', () => {
    it('case-sensitive sort sorts the data in ascending order', () => {
      const data = [
        {[FIELD]: 'z'},
        {[FIELD]: 'a'},
        {[FIELD]: 'Z'},
        {[FIELD]: 'B'}
      ];
      const sortedData = ListMachineUtils.sortDataByFieldAndDirection({useLowerCaseSorting: false})({
        sortField: FIELD,
        sortDirection: ListMachineUtils.ASC,
        data
      });

      expect(sortedData).toStrictEqual([
        {[FIELD]: 'B'},
        {[FIELD]: 'Z'},
        {[FIELD]: 'a'},
        {[FIELD]: 'z'}
      ]);
    });

    it('case-insensitive sort sorts the data in ascending order', () => {
      const data = [
        {[FIELD]: 'z'},
        {[FIELD]: 'a'},
        {[FIELD]: 'Z'},
        {[FIELD]: 'B'}
      ];
      const sortedData = ListMachineUtils.sortDataByFieldAndDirection({useLowerCaseSorting: true})({
        sortField: FIELD,
        sortDirection: ListMachineUtils.ASC,
        data
      });

      expect(sortedData).toStrictEqual([
        {[FIELD]: 'a'},
        {[FIELD]: 'B'},
        {[FIELD]: 'Z'},
        {[FIELD]: 'z'}
      ]);
    });

    it('sorts null as if it\'s an empty string', () => {
      const data = [
        {[FIELD]: "1"},
        {[FIELD]: null}
      ];

      expect(ListMachineUtils.sortDataByFieldAndDirection({useLowerCaseSorting: true})({
        sortField: FIELD,
        sortDirection: ListMachineUtils.ASC,
        data
      })).toStrictEqual([
        {[FIELD]: null},
        {[FIELD]: "1"}
      ]);

      expect(ListMachineUtils.sortDataByFieldAndDirection({useLowerCaseSorting: true})({
        sortField: FIELD,
        sortDirection: ListMachineUtils.DESC,
        data
      })).toStrictEqual([
        {[FIELD]: "1"},
        {[FIELD]: null}
      ]);
    });
  });

  describe('nextSortDirection', () => {
    const nextSortDirection = ListMachineUtils.nextSortDirection(FIELD);

    it('returns ASC when no field is sorted', () => {
      expect(nextSortDirection({
        sortField: null,
        sortDirection: null
      })).toBe(ListMachineUtils.ASC);
    });

    it('returns ASC when another field is sorted', () => {
      expect(nextSortDirection({
        sortField: 'anotherField',
        sortDirection: ListMachineUtils.ASC
      })).toBe(ListMachineUtils.ASC);
    });

    it('returns ASC when trying to sort a descending field', () => {
      expect(nextSortDirection({
        sortField: FIELD,
        sortDirection: ListMachineUtils.DESC
      })).toBe(ListMachineUtils.ASC);
    });

    it('returns DESC when trying to sort an ascending field', () => {
      expect(nextSortDirection({
        sortField: FIELD,
        sortDirection: ListMachineUtils.ASC
      })).toBe(ListMachineUtils.DESC);
    });
  });

  describe('getSortDirection', () => {
    it('returns the sort direction if the field name matches', () => {
      expect(ListMachineUtils.getSortDirection(FIELD, {
        sortField: FIELD,
        sortDirection: ListMachineUtils.DESC
      })).toBe(ListMachineUtils.DESC);
    });

    it('returns null if the field name does not match', () => {
      expect(ListMachineUtils.getSortDirection(FIELD, {
        sortField: 'anotherField',
        sortDirection: ListMachineUtils.DESC
      })).toBeNull();
    });
  });

  describe('hasAnyMatches', () => {
    const values = ['111', 'aaa', 'rrr', '###'];

    it('returns "true" if there is at least one match', () => {
      expect(ListMachineUtils.hasAnyMatches(values, 'a')).toBeTruthy();
      expect(ListMachineUtils.hasAnyMatches([undefined, ...values, null], '11')).toBeTruthy();
    });

    it('returns "false" if there are no matches', () => {
      expect(ListMachineUtils.hasAnyMatches(values, 'b')).toBeFalsy();
    });

    it('returns "true" if the filter is empty', () => {
      expect(ListMachineUtils.hasAnyMatches(values, '')).toBeTruthy();
      expect(ListMachineUtils.hasAnyMatches(values, null)).toBeTruthy();
      expect(ListMachineUtils.hasAnyMatches(values, undefined)).toBeTruthy();
    });

    it('returns "false" if "values" contains empty values', () => {
      expect(ListMachineUtils.hasAnyMatches([null, undefined, ''], 'a')).toBeFalsy();
    });
  });
});

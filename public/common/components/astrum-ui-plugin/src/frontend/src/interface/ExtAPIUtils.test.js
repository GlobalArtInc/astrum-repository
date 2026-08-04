

import ExtAPIUtils from './ExtAPIUtils';

import APIConstants from '../constants/APIConstants';

const {EXT: {SMALL_PAGE_SIZE}} = APIConstants;

describe('ExtAPIUtils', () => {
  it('extractResult', () => {
    const data = 'test1';
    const defaultValue = 'test2';
    const response = {
      data: {
        result: {
          data
        }
      }
    };

    let result = ExtAPIUtils.extractResult(response, defaultValue);
    expect(result).toBe(data);

    result = ExtAPIUtils.extractResult({}, defaultValue);
    expect(result).toBe(defaultValue);

    result = ExtAPIUtils.extractResult({data}, defaultValue);
    expect(result).toBe(defaultValue);

    result = ExtAPIUtils.extractResult([], defaultValue);
    expect(result).toBe(defaultValue);
  });

  it('createRequestBody', () => {
    const params = {
      action: 'test1',
      method: 'test2'
    };
    const result = {
      ...params,
      data: null,
      tid: 1,
      type: 'rpc',
    };
    expect(ExtAPIUtils.createRequestBody(params.action, params.method)).toStrictEqual(result);

    expect(ExtAPIUtils.createRequestBody(params.action, params.method, {}, 2)).toStrictEqual({
      action: params.action,
      method: params.method,
      tid: 2,
      type: result.type,
      data: [{
        limit: SMALL_PAGE_SIZE,
        page: 1,
        start: 0,
      }]
    });
  });

  describe('createData', () => {
    it('returns data value when presents', () => {
      const data = 'TestData';
      const options = {data};
      expect(ExtAPIUtils.createData(options)).toEqual(data);
    });

    it('returns null when options is empty value', () => {
      expect(ExtAPIUtils.createData(null)).toEqual(null);
      expect(ExtAPIUtils.createData(undefined)).toEqual(null);
      expect(ExtAPIUtils.createData('')).toEqual(null);
    });

    describe('pagination', () => {
      const defaultPaginationParams = {
        limit: SMALL_PAGE_SIZE,
        page: 1,
        start: 0,
      };

      it('returns pagination parameters', () => {
        let options = {};
        let result = [defaultPaginationParams];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {};
        result = [defaultPaginationParams];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {page: 2, start: 20, limit: 10};
        result = [options];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {page: 4};
        result = [{...defaultPaginationParams, ...options}];
        expect(ExtAPIUtils.createData(options)).toEqual(result);
      });

      it('returns filter parameters', () => {
        let options = {filterValue: 'test'};
        let result = [{
          ...defaultPaginationParams,
          filter: [{
            property: 'filter',
            value: options.filterValue,
          }],
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {filterValue: 'test', filterField: 'name'};
        result = [{
          ...defaultPaginationParams,
          filter: [{
            property: options.filterField,
            value: options.filterValue,
          }],
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {filterField: 'name'};
        result = [defaultPaginationParams];
        expect(ExtAPIUtils.createData(options)).toEqual(result);
      });

      it('returns unmodified filter parameters when presents', () => {
        const options = {filter: [{property: 'name', value: 'test'}], filterValue: 'test'};
        const result = [{
          ...defaultPaginationParams,
          filter: options.filter,
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);
      });

      it('returns sorting parameters', () => {
        let options = {sortField: 'test'};
        let result = [{
          ...defaultPaginationParams,
          sort: [{
            property: options.sortField,
            direction: 'ASC',
          }],
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {sortField: 'test', sortDirection: 'DESC'};
        result = [{
          ...defaultPaginationParams,
          sort: [{
            property: options.sortField,
            direction: options.sortDirection,
          }],
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);

        options = {sortDirection: 'DESC'};
        result = [defaultPaginationParams];
        expect(ExtAPIUtils.createData(options)).toEqual(result);
      });

      it('returns query parameter when presents', () => {
        const options = {query: 'test'};
        const result = [{
          ...defaultPaginationParams,
          query: options.query
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);
      });

      it('returns all parameters', () => {
        const options = {
          query: 'test1',
          filterValue: 'test3',
          sortField: 'test2',
        };
        const result = [{
          ...defaultPaginationParams,
          query: options.query,
          filter: [{
            property: 'filter',
            value: options.filterValue,
          }],
          sort: [{
            property: options.sortField,
            direction: 'ASC',
          }],
        }];
        expect(ExtAPIUtils.createData(options)).toEqual(result);
      });
    });
  });
});

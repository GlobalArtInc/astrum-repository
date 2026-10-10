
/*global Ext*/

/**
 * Access point for available {AS.util.condition.Condition}s.
 *
 * @since 3.0
 */
Ext.define('AS.Conditions', {
  singleton: true,
  requires: [
    'AS.util.condition.Conjunction',
    'AS.util.condition.Disjunction',
    'AS.util.condition.FormHasRecord',
    'AS.util.condition.FormIs',
    'AS.util.condition.GridHasSelection',
    'AS.util.condition.IsPermitted',
    'AS.util.condition.StoreHasRecords',
    'AS.util.condition.MultiListener',
    'AS.util.condition.WatchState',
    'AS.util.condition.NeverSatisfied',
    'AS.util.condition.HasNoFirewall'
  ],

  /**
   * @param {String} permission permission
   * @returns {AS.util.condition.IsPermitted}
   */
  isPermitted: function (permission) {
    return Ext.create('AS.util.condition.IsPermitted', { permission: permission });
  },

  /**
   * @param {String} store id of store that should have records
   * @returns {AS.util.condition.StoreHasRecords}
   */
  storeHasRecords: function (store) {
    return Ext.create('AS.util.condition.StoreHasRecords', { store: store });
  },

  /**
   * @param {Ext.app.Controller} the controller to use to detect events on the grid
   * @param {String} grid a grid selector as specified by {@link Ext.ComponentQuery#query}
   * @param {Function} [fn] to be called when grid has a selection to perform additional checks on the passed in model
   * @returns {AS.util.condition.GridHasSelection}
   */
  gridHasSelection: function (grid, fn) {
    return Ext.create('AS.util.condition.GridHasSelection', { grid: grid, fn: fn });
  },

  /**
   * @param {String} form a {@link AS.view.SettingsForm} selector as specified by {@link Ext.ComponentQuery#query}
   * @param {Function} [fn] to be called when form has a record to perform additional checks on the passed in model
   * @returns {AS.util.condition.FormHasRecord}
   */
  formHasRecord: function (form, fn) {
    return Ext.create('AS.util.condition.FormHasRecord', { form: form, fn: fn });
  },

  /**
   * @param {Array} [listenerConfigs] An array of objects { observable: o, events:[] }
   * @param {Function} [fn] A function to be called when an event occurs.
   * @returns {AS.util.condition.MultiListener}
   */
  watchEvents: function (listenerConfigs, fn) {
    return Ext.create('AS.util.condition.MultiListener', { fn: fn, listenerConfigs: listenerConfigs });
  },

  /**
   * @param {String|Ext.Component} [form] A selector or a reference to a form
   * @param {Function} [condition] A function to evaluate against the form to fire the satisfied or unsatisfied event
   * @returns {AS.util.condition.FormIs}
   */
  formIs: function (form, condition) {
    return Ext.create('AS.util.condition.FormIs', { form: form, condition: condition });
  },

  /**
   * @param {String} key state value key
   * @param {Function} [fn] An optional function to be called when a state value changes. If not specified, a boolean
   * check against value will be performed
   * @returns {AS.util.condition.WatchState}
   */
  watchState: function (key, fn) {
    return Ext.create('AS.util.condition.WatchState', { key: key, fn: fn });
  },

  /**
   * Takes as parameter {@link AS.util.condition.Condition}s to be AND-ed.
   *
   * @returns {AS.util.condition.Conjunction}
   */
  and: function () {
    return Ext.create('AS.util.condition.Conjunction', { conditions: Array.prototype.slice.call(arguments) });
  },

  /**
   * Takes as parameter {@link AS.util.condition.Condition}s to be OR-ed.
   *
   * @returns {AS.util.condition.Disjunction}
   */
  or: function () {
    return Ext.create('AS.util.condition.Disjunction', { conditions: Array.prototype.slice.call(arguments) });
  },

  /**
   * No-op condition that is never satisfied.
   *
   * @returns {AS.util.condition.NeverSatisfied}
   */
  never: function() {
    return Ext.create('AS.util.condition.NeverSatisfied');
  },

  /**
   * Checks if this Astrum instance does not have firewall
   *
   * @returns
   */
  hasNoFirewall: function() {
    return Ext.create('AS.util.condition.HasNoFirewall');
  }

});


/*global Ext, AS*/

/**
 * Store {@link AS.util.log.Sink} into {@link AS.store.LogEvent} store.
 *
 * @since 3.0
 */
Ext.define('AS.util.log.StoreSink', {
  extend: 'AS.util.log.Sink',
  requires: [
    'AS.Assert'
  ],

  /**
   * Reference to the event store.
   *
   * @private
   * @property {AS.store.LogEvent}
   */
  store: undefined,

  /**
   * Maximum records to retain in the store.
   *
   * @public
   * @property {Number}
   * @readonly
   */
  maxSize: 200,

  /**
   * @constructor
   * @param {AS.store.LogEvent} store
   */
  constructor: function (store) {
    this.store = store;
    this.callParent(arguments);
  },

  /**
   * Customize state.
   *
   * @override
   * @return {Object}
   */
  getState: function() {
    return Ext.apply(this.callParent(), {
      maxSize: this.maxSize
    });
  },

  /**
   * Set the maximum size of the store.
   *
   * @public
   * @param {Number} maxSize
   */
  setMaxSize: function (maxSize) {
    this.maxSize = maxSize;

    // log here should induce shrinkage, nothing more to do
    this.logDebug('Max size:', maxSize);

    this.saveState();
  },

  /**
   * Array of ordered records for shrinking.
   *
   * @private
   * @property {AS.model.LogEvent[]}
   */
  records: [],

  /**
   * @override
   */
  receive: function (event) {
    //<if assert>
    AS.Assert.assert(this.store, 'Store not attached');
    //</if>

    // only 1 record, pick off first
    var record = this.store.add(event)[0];

    // maybe shrink
    this.shrink();

    // track records for shrinkage
    this.records.push(record);
  },

  /**
   * Shrink the store after we breach maximum size.
   *
   * @private
   */
  shrink: function () {
    // calculate number of records to purge
    var remove = this.records.length - this.maxSize;

    // maybe purge records
    if (remove > 0) {
      var purged = this.records.splice(0, remove);
      this.store.remove(purged);
    }
  }
});

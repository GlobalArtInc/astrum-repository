
/*global Ext, NX*/

/**
 * Allows customizable processing of {@link NX.model.LogEvent}.
 *
 * @since 3.0
 */
Ext.define('NX.util.log.Sink', {
  mixins: {
    stateful: 'Ext.state.Stateful',
    logAware: 'NX.LogAware'
  },

  /**
   * Sink enabled.
   *
   * @property {Boolean}
   * @readonly
   */
  enabled: true,

  /**
   * @constructor
   */
  constructor: function () {
    // setup stateful configuration with class-name, these are not technically singletons but are used as such
    this.mixins.stateful.constructor.call(this, {
      stateful: true,
      stateId: this.self.getName()
    });

    this.callParent(arguments);
    this.initState();
  },

  /**
   * @override
   * @return {Object}
   */
  getState: function() {
    return {
      enabled: this.enabled
    };
  },

  /**
   * Toggle enabled.
   *
   * @public
   * @param {boolean} flag
   */
  setEnabled: function (flag) {
    this.enabled = flag;

    //<if debug>
    this.logInfo('Enabled:', flag);
    //</if>

    this.saveState();
  },

  /**
   * @public
   * @param {NX.model.LogEvent} event
   */
  receive: function(event) {
    throw 'abstract-method';
  }
});


/*global Ext*/

/**
 * **{@link Ext.form.action.DirectLoad}** overrides.
 *
 * See: https://support.sencha.com/index.php#ticket-17182
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.action.DirectLoad', {
  override: 'Ext.form.action.DirectLoad',

  /**
   * @override
   * Bail out if form was already destroyed.
   */
  onComplete: function () {
    if (!this.form.isDestroyed) {
      this.callParent(arguments);
    }
  }

});

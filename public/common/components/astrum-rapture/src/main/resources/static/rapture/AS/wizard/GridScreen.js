
/*global Ext, AS*/

/**
 * Wizard grid screen.
 *
 * @since 3.0
 * @abstract
 */
Ext.define('AS.wizard.GridScreen', {
  extend: 'AS.wizard.Screen',
  alias: 'widget.as-wizard-gridscreen',
  requires: [
    'AS.Assert'
  ],

  config: {
    /**
     * @cfg {Object} {@link Ext.grid.Panel} configuration object.
     */
    grid: undefined
  },

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    //<if assert>
    AS.Assert.assert(me.grid, 'Missing required config: grid');
    //</if>

    Ext.applyIf(me.grid, {
      xtype: 'grid'
    });

    me.fields = me.fields || [];
    me.fields.push(me.grid);

    me.callParent(arguments);
  },

  /**
   * @return {Ext.grid.Panel}
   */
  getGrid: function () {
    return this.down('grid');
  }
});

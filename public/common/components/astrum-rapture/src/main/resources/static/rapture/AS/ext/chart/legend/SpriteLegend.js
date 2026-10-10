
/*global Ext, AS*/

/**
 * Fix for issue where ComponentQuery thinks the SpriteLegend is a component.
 *
 * @see https://www.sencha.com/forum/showthread.php?470887-Ext-ComponentQuery-candidate-getItemId-is-not-a-function
 */
Ext.define('AS.ext.chart.legend.SpriteLegend', {
  override: 'Ext.chart.legend.SpriteLegend',

  isXType: function (xtype) {
    return xtype === 'sprite';
  },


  getItemId: function () {
    return this.getId();
  }
});

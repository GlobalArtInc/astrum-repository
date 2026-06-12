
/*global Ext, NX*/

/**
 * Wizard panel.
 *
 * @since 3.0
 * @abstract
 */
Ext.define('NX.wizard.Panel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-wizard-panel',
  requires: [
    'NX.I18n'
  ],

  cls: 'nx-wizard-panel',

  autoScroll: true,
  layout: {
    type: 'vbox',
    align: 'stretch'
  },

  items: {
    xtype: 'container',
    itemId: 'container',
    cls: 'screencontainer',
    frame: true,
    layout: 'card'
  },

  // screen header (title + progress)
  dockedItems: {
    xtype: 'toolbar',
    itemId: 'header',
    cls: 'screenheader',
    dock: 'top',
    items: [
      {
        xtype: 'label',
        itemId: 'title',
        cls: 'title'
      },
      '->',
      {
        xtype: 'label',
        itemId: 'progress',
        cls: 'progress'
      }
    ]
  },

  /**
   * @returns {Ext.container.Container}
   */
  getScreenHeader: function () {
    return this.down('#header');
  },

  /**
   * @param {String} title
   */
  setTitle: function (title) {
    this.getScreenHeader().down('#title').setText(title);
  },

  /**
   * @param {number} current  Current screen number.
   * @param {number} total    Total number of screens.
   */
  setProgress: function (current, total) {
    this.getScreenHeader().down('#progress').setText(NX.I18n.format('Wizard_Screen_Progress', current, total)
    );
  },

  /**
   * @returns {Ext.panel.Panel}
   */
  getScreenContainer: function () {
    return this.down('#container');
  }
});
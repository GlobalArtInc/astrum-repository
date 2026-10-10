
/*global Ext, AS*/

/**
 * Wizard panel.
 *
 * @since 3.0
 * @abstract
 */
Ext.define('AS.wizard.Panel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-wizard-panel',
  requires: [
    'AS.I18n'
  ],

  cls: 'as-wizard-panel',

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
    this.getScreenHeader().down('#progress').setText(AS.I18n.format('Wizard_Screen_Progress', current, total)
    );
  },

  /**
   * @returns {Ext.panel.Panel}
   */
  getScreenContainer: function () {
    return this.down('#container');
  }
});

/*global Ext*/

/**
 * Info entry.
 *
 * @since 3.0
 */
Ext.define('NX.view.info.Entry', {
  extend: 'Ext.Component',
  alias: 'widget.nx-info',
  requires: [
    'Ext.XTemplate'
  ],

  statics: {
    /**
     * Wraps HTML content to indicate it should be rendered without encoding.
     * Use this when passing HTML content to showInfo that should not be encoded.
     * @param {String} html The HTML content to render
     * @return {Object} An object that signals to the template to render raw HTML
     */
    rawHtml: function(html) {
      return { isRawHtml: true, html: html };
    }
  },

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.tpl = Ext.create('Ext.XTemplate', [
      '<div class="nx-info">',
      '<table>',
      '<tpl for=".">',
      '<tr class="nx-info-entry">',
      '<td class="nx-info-entry-name">{name:htmlEncode}</td>',
      '<td class="nx-info-entry-value">{[this.encodeValue(values.value)]}</td>',
      '</tr>',
      '</tpl>',
      '</tr>',
      '</table>',
      '</div>',
      {
        encodeValue: function(v) {
          if (v && v.isRawHtml) {
            return v.html;
          }
          return Ext.htmlEncode(v);
        }
      }
    ]);

    me.callParent();
  },

  /**
   * @public
   * @param {Object} info
   */
  showInfo: function (info) {
    var entries = [];
    Ext.Object.each(info, function (key, value) {
      if (!Ext.isEmpty(value)) {
        entries.push(
            {
              name: key,
              value: value
            }
        );
      }
    });
    if (this.getEl()) {
      this.tpl.overwrite(this.getEl(), entries);

      // Ensure the parent panel updates it's layout
      this.up('panel').updateLayout();
    }
    else {
      this.html = this.tpl.apply(entries);
    }
  }

});

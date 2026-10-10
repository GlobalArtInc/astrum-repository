
/*global Ext*/

/**
 * Renderer helpers.
 *
 * @since 3.0
 */
Ext.define('AS.ext.grid.column.Renderers', {
  requires: [
    'AS.I18n'
  ],
  singleton: true,

  /**
   * Renderer which will use no-data glyph if given value is undefined or null.
   */
  optionalData: function(value) {
    return value ?
        Ext.htmlEncode(value) :
        '<span class="x-fa fa-ban" style="opacity: 0.33;" aria-label="' + Ext.util.Format.htmlEncode(AS.I18n.get('Column_No_Data')) + '"/>';
  },

  /**
   * Converts and renders a given date to locale string.
   */
  formattedDate: function(value) {
    return value ? new Date(value).toLocaleString() : '';
  },

  /**
   * Renderer which will display either 'Allowed' or 'Blocked' with a corresponding glyph depending on the value
   * and will dim the text if the rule is not defined in the record.
   */
  allowedBlocked: function(value, column, record) {
    var maybeDim = (record.get('rule') ? '' : ' opacity: 0.33;') + '"/> ',
        children = record.get('children');
    if (children && children.length > 0) {
      return;
    }

    if (value) {
      return '<span class="x-fa fa-check-circle" style="color: #1C8145;' + maybeDim + Ext.util.Format.htmlEncode(AS.I18n.get('RoutingRules_GlobalRoutingPreview_Grid_Allowed'));
    }
    else {
      return '<span class="x-fa fa-ban" style="color: #C70000;' + maybeDim + Ext.util.Format.htmlEncode(AS.I18n.get('RoutingRules_GlobalRoutingPreview_Grid_Blocked'));
    }
  },

  /**
   * Renderer which will display either the rule defined or a dimmed 'None' with a no-data glyph.
   */
  optionalRule: function(value) {
    return value ?
        Ext.htmlEncode(value) :
        '<span class="x-fa fa-ban" style="opacity: 0.33;" /> ' + Ext.util.Format.htmlEncode(AS.I18n.get('RoutingRules_GlobalRoutingPreview_Grid_None'));
  }
});

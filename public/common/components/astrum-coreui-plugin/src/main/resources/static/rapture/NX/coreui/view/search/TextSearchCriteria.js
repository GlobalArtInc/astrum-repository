
/*global Ext, NX*/

/**
 * Generic text search criteria.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.search.TextSearchCriteria', {
  extend: 'NX.ext.SearchBox',
  alias: 'widget.nx-coreui-searchcriteria-text',
  requires: [
    'NX.I18n'
  ],
  mixins: {
    searchCriteria: 'NX.coreui.view.search.SearchCriteria'
  },

  /**
   * @cfg [removable=false] If search criteria should be removable.
   */
  removable: false,

  // Width must be defined on the class, otherwise overrides won’t be applied correctly
  width: 100,

  triggers: {
    remove: {
      cls:  'nx-form-fa-minus-circle-trigger',
      handler: function() {
        var me = this;
        // Fire event about user removing the search criteria.
        me.fireEvent('criteriaremoved', me);
      }
    }
  },

  filter: function() {
    var me = this;
    if (me.value) {
      return { property: me.criteriaId, value: me.value };
    }
    return undefined;
  },

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.emptyText = NX.I18n.get('Search_TextSearchCriteria_Filter_EmptyText');
    me.padding = '0 5 0 0';
    me.labelAlign = 'top';
    me.labelSeparator = '';

    me.triggers.remove.setHidden(!me.removable);

    me.callParent();
  },

  onValueChange: function(trigger, value) {
    this.setValue(value);
    this.resetOriginalValue();
  }

});

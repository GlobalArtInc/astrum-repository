
/*global Ext, NX*/

/**
 * User search box.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserSearchBox', {
  extend: 'Ext.form.field.Text',
  alias: 'widget.nx-coreui-user-searchbox',
  requires: [
    'Ext.util.KeyNav',
    'NX.I18n'
  ],

  triggers: {
    clear: {
      cls: 'nx-form-fa-times-circle-trigger',
      handler: 'clearSearch',
      hidden: true
    },
    search: {
      cls: 'x-form-search-trigger',
      handler: 'doSearch'
    }
  },

  keyMap: {
    ESC: 'clearSearch',
    ENTER: 'doSearch'
  },

  listeners: {
    change: 'valueChanged'
  },

  width: 320,
  submitValue: false,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.emptyText = NX.I18n.get('User_UserList_Filter_EmptyText');

    me.callParent();
  },

  /**
   * @private
   */
  doSearch: function() {
    var value = this.getValue();
    this.search(value);
  },

  /**
   * Search for value and fires a 'search' event.
   *
   * @public
   * @param value to search for
   */
  search: function(value) {
    var me = this;

    if (value !== me.getValue()) {
      me.setValue(value);
    }
    me.fireEvent('search', me, value);
  },

  /**
   * Clears the search.
   *
   * @public
   */
  clearSearch: function() {
    var me = this;

    if (me.getValue()) {
      me.setValue(undefined);
    }
    me.fireEvent('searchcleared', me);
  },

  valueChanged: function() {
    var clearTrigger = this.getTrigger('clear');
    if (this.getValue()) {
      clearTrigger.show();
    }
    else {
      clearTrigger.hide();
    }
  }

});

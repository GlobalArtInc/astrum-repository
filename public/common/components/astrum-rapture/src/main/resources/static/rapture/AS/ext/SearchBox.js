
/*global Ext*/

/**
 * A search box.
 *
 * @since 3.0
 */
Ext.define('AS.ext.SearchBox', {
  extend: 'Ext.form.field.Text',
  alias: 'widget.as-searchbox',
  requires: [
    'Ext.util.KeyNav'
  ],

  cls: 'as-searchbox',

  emptyText: 'search',
  submitValue: false,

  /**
   * Number of milliseconds to trigger searching.
   *
   * @cfg {Number}
   */
  searchDelay: 1000,

  triggers: {
    clear: {
      cls: 'as-form-fa-times-circle-trigger',
      handler: function() {
        this.clearSearch();
      }
    }
  },

  listeners: {
    change: 'onValueChange',
    keypress: 'updateTriggerVisibility'
  },

  maskOnDisable: false,

  /**
   * @override
   */
  initComponent: function () {
    var config = {
      checkChangeBuffer: this.searchDelay,
      ariaLabel: this.emptyText
    },
    fieldSubTpl = '<span class="as-searchbox-icon x-fa ' + this.iconClass + '"></span>';

    if (this.iconClass) {
      config.inputWrapCls = 'has-icon';

      if (this.fieldSubTpl instanceof Ext.XTemplate) {
        config.fieldSubTpl = [fieldSubTpl].concat(this.fieldSubTpl.html);
        this.fieldSubTpl.destroy();
      } else {
        config.fieldSubTpl = [fieldSubTpl].concat(this.fieldSubTpl);
      }

      config.listeners = {
        afterrender: function() {
          var icon = this.getEl().down('span.as-searchbox-icon');
          if (icon) {
            icon.on('click', this.focus.bind(this));
          }
        }.bind(this)
      };
    }

    Ext.apply(this, config);

    this.callParent(arguments);
  },

  /**
   * @override
   */
  initEvents: function () {
    var me = this;

    me.callParent();

    me.keyNav = new Ext.util.KeyNav({
      target: me.inputEl,
      esc: {
        handler: me.clearSearch,
        scope: me,
        defaultEventAction: false
      },
      enter: {
        handler: me.onEnter,
        scope: me,
        defaultEventAction: false
      },
      scope: me,
      forceKeyDown: true
    });
  },

  /**
   * Search on ENTER.
   *
   * @private
   */
  onEnter: function () {
    var me = this;

    //me.lastValue is used to check for changes in the delayed checkchanges task, so we fake it out here
    //otherwise, the onValueChange will get triggered regardless when timeout occurs
    //(causing undesired page transition if page is changed prior to this delayed check)
    me.lastValue = me.getValue();
    me.search(me.lastValue);
    me.resetOriginalValue();
  },

  /**
   * Trigger search.
   *
   * @private
   */
  onValueChange: function (trigger, value) {
    var me = this;

    if (value) {
      me.search(value);
    }
    else {
      me.clearSearch();
    }
    me.resetOriginalValue();
  },

  /**
   * Search for value and fires a 'search' event.
   *
   * @public
   * @param value to search for
   */
  search: function (value) {
    var me = this;

    if (value !== me.getValue()) {
      me.setValue(value);
    }
    else {
      if (me.fireEvent('beforesearch', me)) {
        me.fireEvent('search', me, value);
      }
    }
  },

  /**
   * Clears the search and fires a 'searchcleared' event.
   *
   * @public
   */
  clearSearch: function () {
    var me = this;

    if (me.getValue()) {
      me.setValue(undefined);
    }
    me.fireEvent('searchcleared', me);
  },

  onEnable: function() {
    this.show();
    this.callParent();
  },

  onDisable: function() {
    this.hide();
    this.callParent();
  }

});

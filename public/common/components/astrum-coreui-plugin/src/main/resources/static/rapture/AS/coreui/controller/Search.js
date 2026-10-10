
/*global Ext, AS*/

/**
 * Search controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Search', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.view.info.Panel',
    'AS.view.info.Entry',
    'AS.Bookmarks',
    'AS.Conditions',
    'AS.Permissions',
    'AS.I18n',
    'AS.State',
    'AS.coreui.util.BrowseableFormats'
  ],
  masters: [
    'as-coreui-searchfeature as-coreui-search-result-list',
    'as-coreui-searchfeature as-coreui-component-asset-list'
  ],
  stores: [
    'ComponentAsset',
    'SearchFilter',
    'SearchCriteria',
    'SearchResult'
  ],
  models: [
    'Asset',
    'Component',
    'SearchFilter'
  ],

  views: [
    'search.SearchFeature',
    'search.SearchResultList',
    'search.TextSearchCriteria',
    'search.SaveSearchFilter'
  ],

  refs: [
    { ref: 'feature', selector: 'as-coreui-searchfeature' },
    { ref: 'searchResult', selector: 'as-coreui-searchfeature as-coreui-search-result-list' },
    { ref: 'componentDetails', selector: 'as-coreui-searchfeature as-coreui-component-details' },
    { ref: 'assetList', selector: 'as-coreui-searchfeature as-coreui-component-asset-list' },
    { ref: 'quickSearch', selector: 'as-header-panel #quicksearch' },
    { ref: 'info', selector: 'as-coreui-searchfeature #info' },
    { ref: 'warning', selector: 'as-coreui-searchfeature #warning' }
  ],

  timedOutMessage: null,
  limitMessage: null,

  /**
   * @override
   */
  init: function() {
    var me = this;
    me.callParent();
    me.getApplication().getIconController().addIcons({
      'search-default': {
        file: 'magnifier.png',
        variants: ['x16', 'x32']
      },
      'search-component': {
        file: 'box_front.png',
        variants: ['x16', 'x32']
      },
      'search-component-detail': {
        file: 'box_front_open.png',
        variants: ['x16', 'x32']
      },
      'search-folder': {
        file: 'folder_search.png',
        variants: ['x16', 'x32']
      },
      'search-saved': {
        file: 'magnifier.png',
        variants: ['x16', 'x32']
      },
      'message-primary': {
        file: 'information.png',
        variants: ['x16', 'x32'],
        preload: true
      },
      'message-danger': {
        file: 'exclamation.png',
        variants: ['x16', 'x32'],
        preload: true
      }
    });
    me.registerFilter([
      {
        id: 'keyword',
        name: 'Keyword',
        text: 'Keyword',
        description: 'Search for components by keyword',
        readOnly: true,
        criterias: [
          { id: 'keyword' }
        ]
      },
      {
        id: 'custom',
        name: 'Custom',
        text: AS.I18n.get('Search_Custom_Text'),
        description: AS.I18n.get('Search_Custom_Description'),
        readOnly: true
      }
    ], me);
    me.getStore('SearchFilter').each(function(model) {
      me.registerFeature(model, me);
    });
    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        },
        '#State': {
          userchanged: me.onUserChanged
        },
        '#Permissions': {
          changed: me.onPermissionsChange
        }
      },
      component: {
        'as-coreui-searchfeature as-coreui-search-result-list': {
          beforerender: me.onBeforeRender,
          render: me.showHideResponseMessage
        },
        'as-coreui-searchfeature': {
          afterrender: me.initCriterias
        },
        'as-coreui-searchfeature menuitem[action=add]': {
          click: me.addCriteria
        },
        'as-coreui-searchfeature component[searchCriteria=true]': {
          search: me.onSearchCriteriaChange,
          searchcleared: me.onSearchCriteriaChange,
          criteriaremoved: me.removeCriteria
        },
        'as-coreui-searchfeature button[action=save]': {
          click: me.showSaveSearchFilterWindow
        },
        'as-coreui-search-save button[action=add]': {
          click: me.saveSearchFilter
        },
        'as-main #quicksearch': {
          afterrender: me.bindQuickSearch,
          search: me.onQuickSearch,
          searchcleared: me.onQuickSearch
        }
      },
      store: {
        '#SearchResult': {
          load: me.setResponseMessage
        }
      }
    });
  },

  /**
   * Update search results after user changed.
   *
   * @private
   */
  onUserChanged: function (user, oldUser) {
    var me = this,
        searchResultStore = me.getSearchResultStore();
    if (!user) {
      if (searchResultStore !== null) {
        searchResultStore.clearFilter(true);
      }
    }
  },

  /**
   * Update search results after permissions changed.
   *
   * @private
   */
  onPermissionsChange: function () {
    var me = this,
        searchResultStore = me.getSearchResultStore();
    if (AS.Permissions.check('astrum:search:read')
        && searchResultStore !== undefined
        && searchResultStore !== null) {
      searchResultStore.load();
    }
  },

  /**
   * @public
   * Register a set of criterias.
   * @param {Array/Object} criterias to be registered
   * @param {Ext.util.Observable} [owner] to be watched to automatically unregister the criterias if owner is destroyed
   */
  registerCriteria: function(criterias, owner) {
    var me = this,
        models;

    models = me.getStore('SearchCriteria').add(criterias);
    if (owner) {
      owner.on('destroy', function() {
        me.getStore('SearchCriteria').remove(models);
      }, me);
    }
  },

  /**
   * @public
   * Register a set of filters.
   * @param {Array/Object} filters to be registered
   * @param {Ext.util.Observable} [owner] to be watched to automatically unregister the criterias if owner is destroyed
   */
  registerFilter: function(filters, owner) {
    var me = this;

    Ext.each(Ext.Array.from(filters), function(filter) {
      me.registerFeature(me.getSearchFilterModel().create(filter), owner);
    });
  },

  isBrowseableBasedOnFormatCriteria: function(model) {
    var formatCriteria = Ext.Array.from(model.get('criterias')).filter(function(item) {
      return item.id === 'format';
    });

    if (formatCriteria.length > 0) {
      return AS.coreui.util.BrowseableFormats.check(formatCriteria[0].value);
    }
    else {
      return true;
    }
  },

  /**
   * @private
   * Register feature for model.
   * @param {AS.coreui.model.SearchFilter} model to be registered
   * @param {Ext.util.Observable} [owner] to be watched to automatically unregister the criterias if owner is destroyed
   */
  registerFeature: function(model, owner) {
    var me = this;

    if (model.getId() === 'keyword') {
      me.getApplication().getFeaturesController().registerFeature({
        mode: 'browse',
        path: '/Search',
        text: AS.I18n.get('Search_Text'),
        description: AS.I18n.get('Search_Description'),
        group: true,
        view: { xtype: 'as-coreui-searchfeature', searchFilter: model, bookmarkEnding: '' },
        iconCls: 'x-fa fa-search',
        weight: 20,
        expanded: false,
        visible: function() {
          return AS.Permissions.check('astrum:search:read');
        }
      }, owner);
    }
    else {
      me.getApplication().getFeaturesController().registerFeature({
        mode: 'browse',
        path: '/Search/' + (model.get('readOnly') ? '' : 'Saved/') + model.get('name'),
        view: { xtype: 'as-coreui-searchfeature', searchFilter: model, bookmarkEnding: '/' + model.getId() },
        iconCls: 'x-fa fa-search',
        text: model.get('text'),
        description: model.get('description'),
        authenticationRequired: false,
        visible: function() {
          return AS.Permissions.check('astrum:search:read') && me.isBrowseableBasedOnFormatCriteria(model);
        }
      }, owner);
    }
  },

  /**
   * @private
   * Show quick search when user has 'astrum:search:read' permission.
   */
  bindQuickSearch: function(quickSearch) {
    quickSearch.up('panel').mon(
        AS.Conditions.isPermitted('astrum:search:read'),
        {
          satisfied: quickSearch.show,
          unsatisfied: quickSearch.hide,
          scope: quickSearch
        }
    );
  },

  /**
   * @private
   *
   * Store a message to be displayed along with the store results
   */
  setResponseMessage: function() {
    var store = this.getSearchResultStore(),
        rawData = store.proxy.reader.rawData,
        limited = rawData && rawData.limited,
        format = Ext.util.Format.numberRenderer('0,000');

    this.timedOutMessage = null;

    if (limited) {
      this.limitMessage = AS.I18n.format('Search_Results_Limit_Message', format(rawData.total), format(rawData.unlimitedTotal));
    }
    else {
      this.limitMessage = null;
    }

    this.showHideResponseMessage();

    // No filter keys indicates the user navigated to the page and this event was for the initial store
    if (Object.keys(store.getFilters().map) && window.aptrinsic) {
        var properties = { results : store.getCount() };

        // Indicate which filters were used
        Object.keys(store.getFilters().map).forEach(function (key) { properties[key] = true; });

        // Set the format
        var formatFilter = store.getFilters().map['format'];
        properties.format = formatFilter ? formatFilter['_value'] :  'keyword';

        aptrinsic('track', 'Search', properties);
    }
  },

  /**
   * @private
   *
   * Show/hide the response messages
   */
  showHideResponseMessage: function() {
    var warning = this.getWarning(),
        info = this.getInfo();

    if (!warning || !info) {
      return;
    }

    if (this.timedOutMessage && warning) {
      warning.setTitle(this.timedOutMessage);
      warning.show();
    }
    else {
      warning.hide();
    }

    if (this.limitMessage && info) {
      info.setTitle(this.limitMessage);
      info.show();
    }
    else {
      info.hide();
    }
  },

  /**
   * @private
   * Initialize search criterias (filters) based on filter definition and bookmarked criterias.
   */
  initCriterias: function() {
    var me = this,
        searchPanel = me.getFeature(),
        searchFilter = searchPanel.searchFilter,
        searchCriteriaPanel = searchPanel.down('#criteria'),
        searchResultStore = me.getSearchResultStore(),
        searchCriteriaStore = me.getSearchCriteriaStore(),
        addCriteriaMenu = [],
        criterias = {}, criteriasPerGroup = {};

    searchCriteriaPanel.removeAll();

    if (searchFilter && searchFilter.get('criterias')) {
      Ext.Array.each(Ext.Array.from(searchFilter.get('criterias')), function(criteria) {
        criterias[criteria['id']] = { value: criteria['value'], hidden: criteria['hidden'] };
      });
    }
    searchResultStore.getFilters().items.forEach(function(filter) {
      var existingCriteria = criterias[filter.config.id];
      if (existingCriteria) {
        existingCriteria['value'] = filter.config.value;
      }
      else {
        criterias[filter.config.id] = { value: filter.config.value, removable: true };
      }
    });

    Ext.Object.each(criterias, function(id, criteria) {
      var criteriaModel = searchCriteriaStore.getById(id);

      if (criteriaModel) {
        var cmpClass = Ext.ClassManager.getByAlias('widget.as-coreui-searchcriteria-' + criteriaModel.getId());
        if (!cmpClass) {
          cmpClass = Ext.ClassManager.getByAlias('widget.as-coreui-searchcriteria-text');
        }
        searchCriteriaPanel.add(cmpClass.create(Ext.apply(Ext.clone(criteriaModel.get('config')), {
          criteriaId: criteriaModel.getId(),
          value: criteria['value'],
          hidden: criteria['hidden'],
          removable: criteria['removable']
        })));
      }
    });

    searchCriteriaStore.each(function(criteria) {
      // skip tags entry if not running PRO
      if (criteria.getId() === 'tags' && !me.isTaggingEnabled()) {
        return;
      }

      var addTo = addCriteriaMenu,
          group = criteria.get('group'),
          format = criteria.get('config').format;

      if (!format || AS.coreui.util.BrowseableFormats.check(format)) {
        if (group) {
          if (!criteriasPerGroup[group]) {
            criteriasPerGroup[group] = [];
          }
          addTo = criteriasPerGroup[group];
        }
        addTo.push({
          text: criteria.get('config').fieldLabel,
          criteria: criteria,
          criteriaId: criteria.getId(),
          action: 'add',
          disabled: Ext.isDefined(criterias[criteria.getId()])
        });
      }
    });
    Ext.Object.each(criteriasPerGroup, function(key, value) {
      addCriteriaMenu.push({
        text: key,
        menu: value
      });
    });

    searchCriteriaPanel.add({
      xtype: 'button',
      cls: 'more-criteria',
      itemId: 'addButton',
      text: AS.I18n.get('Search_More_Text'),
      iconCls: 'x-fa fa-plus-circle',
      menu: addCriteriaMenu
    });
  },

  /**
   * @private
   * Sets the store filters based on the Bookmark
   */
  loadBookmark: function() {
    var me = this,
        searchFilter = me.getFeature().searchFilter,
        bookmarkSegments = AS.Bookmarks.getBookmark().getSegments(),
        store = me.getStore('SearchResult'),
        bookmarkValues = {},
        criterias = {},
        filterSegments,
        queryIndex,
        pair;

    // Extract the filter object from the URI
    if (bookmarkSegments && bookmarkSegments.length) {
      queryIndex = bookmarkSegments[0].indexOf('=');
      if (queryIndex !== -1) {
        filterSegments = decodeURIComponent(bookmarkSegments[0].slice(queryIndex + 1)).split(' AND ');
        for (var i = 0; i < filterSegments.length; ++i) {
          pair = filterSegments[i].split('=');
          bookmarkValues[pair[0]] = pair[1];
        }
      }
    }

    // From the search type (e.g. maven, nuget, custom)
    if (searchFilter) {
      if (searchFilter.id !== 'keyword' && searchFilter.id !== 'custom') {
        store.proxy.setExtraParam('formatSearch', true);
      } else {
        store.proxy.setExtraParam('formatSearch', false);
      }
      if (searchFilter.get('criterias')) {
        searchFilter.get('criterias').forEach(function(criteria) {
          if (criteria.value) {
            criterias[criteria.id] = {value: criteria.value, hidden: criteria.hidden};
          }
        });
      }
    }

    Ext.Object.each(bookmarkValues, function(key, value) {
      var existingCriteria = criterias[key];
      if (existingCriteria) {
        existingCriteria['value'] = value;
      }
      else {
        criterias[key] = { value: value, removable: true };
      }
    });

    const filters = Object.entries(criterias).map(function(criteriaEntry) {
      return {
        id: criteriaEntry[0],
        property: criteriaEntry[0],
        value: criteriaEntry[1].value
      }
    }).filter(function(criteria) {
      return criteria.value;
    });

    me.applyFilters(filters, false);
  },

  /**
   * @private
   * @returns {boolean} whether tagging feature is available
   */
  isTaggingEnabled: function() {
    return AS.State.getUser() && AS.Permissions.check('astrum:tags:read') && ('PRO' === AS.State.getEdition());
  },

  /**
   * @override
   */
  getDescription: function(model) {
    return model.get('name');
  },

  /**
   * @private
   * Add a criteria.
   * @param menuitem selected criteria menu item
   */
  addCriteria: function(menuitem) {
    var searchPanel = this.getFeature(),
        searchCriteriaPanel = searchPanel.down('#criteria'),
        addButton = searchCriteriaPanel.down('#addButton'),
        searchInfo = searchPanel.down('#searchInfo'),
        criteria = menuitem.criteria,
        cmpClass = Ext.ClassManager.getByAlias('widget.as-coreui-searchcriteria-' + criteria.getId()),
        cmp;

    menuitem.disable();
    if (!cmpClass) {
      cmpClass = Ext.ClassManager.getByAlias('widget.as-coreui-searchcriteria-text');
    }
    searchCriteriaPanel.remove(addButton, false);
    cmp = cmpClass.create(
        Ext.apply(Ext.clone(criteria.get('config')), { criteriaId: criteria.getId(), value: undefined, removable: true })
    );
    searchCriteriaPanel.add(cmp);
    cmp.focus();
    searchCriteriaPanel.add(addButton);

    if (criteria.getId() === 'keyword' && searchInfo) {
      searchInfo.setTitle(AS.I18n.get('Search_KeywordSearchRestrictions'));
    }
  },

  /**
   * @private
   * Remove a criteria.
   * @param searchCriteria removed search criteria
   */
  removeCriteria: function(searchCriteria) {
    var searchCriteriaPanel = this.getFeature().down('#criteria');

    searchCriteriaPanel.remove(searchCriteria);
    searchCriteriaPanel.down('menuitem[criteriaId=' + searchCriteria.criteriaId + ']').enable();
    this.applyFilter({ criteriaId: searchCriteria.criteriaId }, true);
  },

  /**
   * @private
   * Start searching on criteria value changed.
   * @param searchCriteria changed criteria
   */
  onSearchCriteriaChange: function(searchCriteria) {
    this.applyFilter(searchCriteria, true);
  },

  /**
   * @override
   * Search on refresh.
   */
  loadStores: function() {
    var me = this;
    if (me.getFeature()) {
      if (me.currentIndex === 0) {
        me.getSearchResult().getStore().load();
      }
      if (me.currentIndex >= 1) {
        me.getAssetList().getStore().load(function () {
          me.reselect();
        });
      }
    }
  },

  /**
   * @private
   * Load stores based on the bookmarked URL
   */
  onBeforeRender: function() {
    var me = this,
        bookmark = AS.Bookmarks.getBookmark(),
        list_ids = bookmark.getSegments().slice(1),
        searchResultStore = me.getSearchResult().getStore(),
        componentModel;

    searchResultStore.clearFilter(true);
    me.loadBookmark();

    searchResultStore.load(function() {
      // If no search filter has been specified, don't load any stores
      if (!searchResultStore.getFilters().length) {
        return;
      }

      // Load the asset detail view
      if (list_ids[1]) {
        componentModel = searchResultStore.getById(decodeURIComponent(list_ids[0]));
        me.onModelChanged(0, componentModel);
        me.onSearchResultSelection(componentModel);
        me.getAssetList().getStore().load(function () {
          me.reselect();
        });
      }
      // Load the asset list view
      else if (list_ids[0]) {
        me.reselect();
      }
    });
  },

  /**
   * @private
   * Synchronize store filters with search criteria.
   * @param searchCriteria criteria to be synced
   * @param performSearchAfter if filter should be applied on store ( = remote call)
   */
  applyFilter: function(searchCriteria, performSearchAfter) {
    const allCriteria = this.getFeature().query('#criteria component[criteriaId]');
    const filters = allCriteria.filter(function(criteria) {
      return criteria.getValue();
    }).map(function (criteria) {
      return {
        id: criteria.criteriaId,
        property: criteria.criteriaId,
        value: criteria.getValue()
      };
    });

    this.applyFilters(filters, performSearchAfter)
  },

  /**
   * @private
   * Applies a set of filters to the SearchResult store.
   * @param filters {[{property:string, value:string}]}
   * @param performSearchAfter {boolean} if the filters should be applied to the store (performs a remote search)
   */
  applyFilters: function (filters, performSearchAfter) {
    const SUPPRESS_EVENTS = true;
    const searchResultStore = this.getStore('SearchResult');
    this.getSearchResult().getSelectionModel().deselectAll();

    searchResultStore.clearFilter(SUPPRESS_EVENTS);
    filters.forEach(function(filter) {
      // Setting all filters all at once causes bugs.
      // Fortunately, only one search is generated if we add all the filters and then apply them by loading the store.
      searchResultStore.addFilter(filter, SUPPRESS_EVENTS);
    });

    if (performSearchAfter) {
      searchResultStore.load();
      this.onSearchResultSelection(null);
      this.bookmarkFilters();
    }
  },

  /**
   * @override
   * When a list managed by this controller is clicked, route the event to the proper handler
   */
  onSelection: function(list, model) {
    var modelType = list.getStore().model;

    if (modelType === this.getComponentModel()) {
      this.onSearchResultSelection(model);
    }
    else if (modelType === this.getAssetModel()) {
      this.onAssetSelection(model);
    }
  },

  /**
   * @private
   * Show details and load assets of selected component.
   * @param {AS.coreui.model.Component} model selected component
   */
  onSearchResultSelection: function(model) {
    var me = this;

    me.getComponentDetails().setComponentModel(model);
    me.getAssetList().setComponentModel(model);
  },

  /**
   * Load asset container for selected asset
   *
   * @private
   * @param {AS.coreui.model.Asset} model selected asset
   */
  onAssetSelection: function(model) {
    this.getController('AS.coreui.controller.Assets').updateAssetContainer(null, null, null, model);
  },

  /**
   * @private
   * Show "Save Search Filter" window.
   */
  showSaveSearchFilterWindow: function() {
    Ext.widget('as-coreui-search-save');
  },

  /**
   * @private
   * Save a search filter.
   * @param {Ext.button.Button} button 'Add' button from "Save Search Filter"
   */
  saveSearchFilter: function(button) {
    var me = this,
        win = button.up('window'),
        values = button.up('form').getValues(),
        criterias = [],
        model;

    Ext.Array.each(Ext.ComponentQuery.query('as-coreui-searchfeature component[searchCriteria=true]'), function(cmp) {
      criterias.push({
        id: cmp.criteriaId,
        value: cmp.getValue(),
        hidden: cmp.hidden
      });
    });

    model = me.getSearchFilterModel().create(Ext.apply(values, {
      id: values.name,
      criterias: criterias,
      readOnly: false
    }));

    me.getStore('SearchFilter').add(model);

    me.getApplication().getFeaturesController().registerFeature({
      path: '/Search/' + (model.get('readOnly') ? '' : 'Saved/') + model.get('name'),
      mode: 'browse',
      view: { xtype: 'as-coreui-searchfeature', searchFilter: model },
      iconName: 'search-saved',
      description: model.get('description'),
      authenticationRequired: false
    }, me);

    me.getController('Menu').refreshTree();
    AS.Bookmarks.navigateTo(AS.Bookmarks.fromToken('browse/search/saved/' + model.get('name')));

    win.close();
  },

  /**
   * @private
   * Bookmark search values.
   */
  bookmarkFilters: function() {
    var filterArray = [],
        firstSegment, segments;

    // Remove any pre-existing query string
    firstSegment = AS.Bookmarks.getBookmark().getSegment(0);
    if (firstSegment.indexOf('=') !== -1) {
      firstSegment = firstSegment.slice(0, firstSegment.indexOf('='));
    }

    // Add each criteria to the filter object
    Ext.Array.each(Ext.ComponentQuery.query('as-coreui-searchfeature component[searchCriteria=true]'), function(cmp) {
      if (cmp.getValue() && !cmp.isHidden()) {
        filterArray.push(cmp.criteriaId + '=' + cmp.getValue());
      }
    });

    // Stringify and url encode the filter object, then bookmark it
    segments = [firstSegment + "=" + encodeURIComponent(filterArray.join(' AND '))];
    AS.Bookmarks.bookmark(AS.Bookmarks.fromSegments(segments), this);
  },

  /**
   * @private
   * @param {AS.ext.SearchBox} quickSearch search box
   * @param {String} searchValue search value
   */
  onQuickSearch: function(quickSearch, searchValue) {
    var me = this,
        searchFeature = me.getFeature();

    if (!searchFeature || (searchFeature.searchFilter.getId() !== 'keyword')) {
      if (searchValue) {
        AS.Bookmarks.navigateTo(
            AS.Bookmarks.fromToken('browse/search=' + encodeURIComponent('keyword=' + searchValue)),
            me
        );
      }
    }
    else {
      me.showChild(0, true);
      searchFeature.down('#criteria component[criteriaId=keyword]').setValue(searchValue);
      searchFeature.down('#criteria component[criteriaId=keyword]').search(searchValue);
    }
  },

  onNavigate: function() {
    var bookmark = AS.Bookmarks.getBookmark();
    var token = bookmark && bookmark.getToken ? bookmark.getToken() : null;
    if (token && token.includes('browse/search')) {
      setTimeout(function(){
        this.reselect()
      }.bind(this), 100);
    }
  }

});

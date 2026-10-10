/*
 * Russian translations of the ExtJS framework's own texts: dates, number formats, paging, grid menus, field
 * validation and dialog buttons. Appended to the page's i18n script after the framework is loaded; classes the
 * application does not include are skipped.
 */
(function () {
  function override(className, members) {
    var cls = Ext.ClassManager.get(className);
    if (cls) {
      Ext.override(cls, members);
    }
  }

  var shortMonthNames = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн', 'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];

  if (Ext.Date) {
    Ext.Date.monthNames = ['Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь', 'Июль', 'Август', 'Сентябрь',
      'Октябрь', 'Ноябрь', 'Декабрь'];
    Ext.Date.getShortMonthName = function (month) {
      return shortMonthNames[month];
    };
    Ext.Date.monthNumbers = {};
    Ext.each(shortMonthNames, function (name, index) {
      Ext.Date.monthNumbers[name] = index;
    });
    Ext.Date.getMonthNumber = function (name) {
      return Ext.Date.monthNumbers[name.substring(0, 1).toUpperCase() + name.substring(1, 3).toLowerCase()];
    };
    Ext.Date.dayNames = ['Воскресенье', 'Понедельник', 'Вторник', 'Среда', 'Четверг', 'Пятница', 'Суббота'];
    Ext.Date.getShortDayName = function (day) {
      return ['Вс', 'Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб'][day];
    };
    Ext.Date.firstDayOfWeek = 1;
    Ext.Date.defaultFormat = 'd.m.Y';
    Ext.Date.defaultTimeFormat = 'H:i';
  }

  if (Ext.util && Ext.util.Format) {
    Ext.apply(Ext.util.Format, {
      thousandSeparator: ' ',
      decimalSeparator: ',',
      currencySign: '₽',
      currencyAtEnd: true,
      currencySpacer: ' ',
      defaultDateFormat: 'd.m.Y'
    });
  }

  override('Ext.LoadMask', {
    msg: 'Загрузка...'
  });

  override('Ext.view.AbstractView', {
    loadingText: 'Загрузка...'
  });

  override('Ext.window.MessageBox', {
    buttonText: {
      ok: 'OK',
      yes: 'Да',
      no: 'Нет',
      cancel: 'Отмена'
    }
  });

  override('Ext.tab.Tab', {
    closeText: 'Закрыть вкладку'
  });

  override('Ext.form.Basic', {
    waitTitle: 'Пожалуйста, подождите...'
  });

  override('Ext.form.field.Base', {
    invalidText: 'Значение в этом поле неверное',
    ariaErrorText: 'Ошибка ввода. {0}.'
  });

  override('Ext.form.FieldContainer', {
    ariaErrorText: 'Ошибка ввода. {0}.'
  });

  override('Ext.form.field.Text', {
    minLengthText: 'Минимальная длина этого поля — {0}',
    maxLengthText: 'Максимальная длина этого поля — {0}',
    blankText: 'Это поле обязательно для заполнения'
  });

  override('Ext.form.field.Number', {
    minText: 'Значение этого поля не может быть меньше {0}',
    maxText: 'Значение этого поля не может быть больше {0}',
    nanText: '{0} не является числом',
    negativeText: 'Значение не может быть отрицательным'
  });

  override('Ext.form.field.Date', {
    disabledDaysText: 'Недоступно',
    disabledDatesText: 'Недоступно',
    minText: 'Дата в этом поле должна быть позже {0}',
    maxText: 'Дата в этом поле должна быть раньше {0}',
    invalidText: '{0} не является правильной датой — дата должна быть в формате {1}',
    formatText: 'Ожидаемый формат даты: {0}.',
    startDay: 1
  });

  override('Ext.form.field.Time', {
    minText: 'Время в этом поле должно быть позже {0}',
    maxText: 'Время в этом поле должно быть раньше {0}',
    invalidText: '{0} не является правильным временем',
    formatText: 'Ожидаемый формат времени: ЧЧ:ММ'
  });

  override('Ext.form.field.File', {
    buttonText: 'Обзор...'
  });

  override('Ext.form.CheckboxGroup', {
    blankText: 'Выберите хотя бы один элемент в этой группе'
  });

  override('Ext.form.RadioGroup', {
    blankText: 'Выберите один элемент в этой группе'
  });

  if (Ext.form && Ext.form.field && Ext.form.field.VTypes) {
    Ext.apply(Ext.form.field.VTypes, {
      emailText: 'Это поле должно содержать адрес электронной почты в формате "user@example.com"',
      urlText: 'Это поле должно содержать URL в формате "http://www.example.com"',
      alphaText: 'Это поле может содержать только латинские буквы и символ подчёркивания "_"',
      alphanumText: 'Это поле может содержать только латинские буквы, цифры и символ подчёркивания "_"'
    });
  }

  override('Ext.picker.Date', {
    todayText: 'Сегодня',
    minText: 'Эта дата раньше минимальной',
    maxText: 'Эта дата позже максимальной',
    disabledDaysText: '',
    disabledDatesText: '',
    nextText: 'Следующий месяц (Control+Вправо)',
    prevText: 'Предыдущий месяц (Control+Влево)',
    monthYearText: 'Выбор месяца (Control+Вверх/Вниз — выбор года)',
    todayTip: '{0} (Пробел)',
    format: 'd.m.Y',
    startDay: 1
  });

  override('Ext.picker.Month', {
    okText: '&#160;OK&#160;',
    cancelText: 'Отмена'
  });

  override('Ext.toolbar.Paging', {
    beforePageText: 'Страница',
    afterPageText: 'из {0}',
    firstText: 'Первая страница',
    prevText: 'Предыдущая страница',
    nextText: 'Следующая страница',
    lastText: 'Последняя страница',
    refreshText: 'Обновить',
    displayMsg: 'Записи {0}–{1} из {2}',
    emptyMsg: 'Нет данных для отображения'
  });

  override('Ext.grid.header.Container', {
    sortAscText: 'Сортировать по возрастанию',
    sortDescText: 'Сортировать по убыванию',
    sortClearText: 'Сбросить сортировку',
    columnsText: 'Столбцы'
  });

  override('Ext.grid.locking.Lockable', {
    lockText: 'Закрепить столбец',
    unlockText: 'Открепить столбец'
  });

  override('Ext.grid.feature.Grouping', {
    emptyGroupText: '(Пусто)',
    groupByText: 'Группировать по этому полю',
    showGroupsText: 'Показывать по группам'
  });

  override('Ext.grid.PropertyColumnModel', {
    nameText: 'Название',
    valueText: 'Значение',
    dateFormat: 'd.m.Y',
    trueText: 'да',
    falseText: 'нет'
  });

  override('Ext.grid.column.Column', {
    dirtyText: 'Значение ячейки изменено'
  });

  override('Ext.grid.column.Boolean', {
    trueText: 'да',
    falseText: 'нет'
  });

  override('Ext.grid.RowEditor', {
    saveBtnText: 'Сохранить',
    cancelBtnText: 'Отмена',
    errorsText: 'Ошибки',
    dirtyText: 'Сохраните или отмените изменения'
  });

  override('Ext.grid.filters.Filters', {
    menuFilterText: 'Фильтры'
  });

  override('Ext.grid.filters.filter.Boolean', {
    yesText: 'Да',
    noText: 'Нет'
  });

  override('Ext.grid.filters.filter.List', {
    loadingText: 'Загрузка...'
  });

  override('Ext.grid.filters.filter.Number', {
    emptyText: 'Введите число...'
  });

  override('Ext.grid.filters.filter.String', {
    emptyText: 'Введите текст фильтра...'
  });

  override('Ext.view.MultiSelectorSearch', {
    searchText: 'Поиск...'
  });

  override('Ext.view.MultiSelector', {
    emptyText: 'Ничего не выбрано',
    removeRowTip: 'Удалить элемент',
    addToolText: 'Найти элементы для добавления'
  });
}());

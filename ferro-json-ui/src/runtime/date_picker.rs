pub(super) const SOURCE: &str = r#"
    // ── Date + time picker ────────────────────────────────────────────────────
    //
    // Progressive enhancement over [data-date-picker] wrappers. Opens a native
    // <dialog> calendar grid in Italian Monday-first locale with WAI-ARIA
    // arrow-key grid navigation. Time = keyboard-navigable listbox via the
    // shared createListboxEngine (D-05). 15-minute slot granularity 06:00-22:00.
    //
    // Markup contract:
    //   [data-date-picker]              — wrapper
    //   [data-date-picker-native]       — <input> form value carrier (YYYY-MM-DD or datetime-local)
    //   [data-date-picker-trigger]      — <button> that opens the dialog
    //   [data-date-picker-dialog]       — <dialog> element
    //
    // The runtime builds the calendar grid and time list inside the dialog on
    // first open; subsequent opens re-render the grid for the stored month.
    //
    // Security (T-249-03-02): trigger label and grid cell text set via
    // textContent / setAttribute only — never innerHTML for user data.
    // Native input value is a browser-validated date string.
    //
    // Idempotent via dataset.datePickerInit guard. Re-scans on fjui:navigated.

    var MONTHS_IT = ['gen','feb','mar','apr','mag','giu','lug','ago','set','ott','nov','dic'];
    var MONTH_NAMES_IT = ['Gennaio','Febbraio','Marzo','Aprile','Maggio','Giugno',
                          'Luglio','Agosto','Settembre','Ottobre','Novembre','Dicembre'];
    var DAYS_SHORT_IT = ['Lu','Ma','Me','Gi','Ve','Sa','Do'];

    function setupDatePicker() {
        var els = document.querySelectorAll('[data-date-picker]');
        for (var i = 0; i < els.length; i++) {
            if (els[i].dataset.datePickerInit) continue;
            els[i].dataset.datePickerInit = '1';
            try { attachDatePickerBehavior(els[i]); } catch (_) {}
        }

        document.addEventListener('fjui:navigated', function() {
            var newEls = document.querySelectorAll('[data-date-picker]');
            for (var i = 0; i < newEls.length; i++) {
                if (newEls[i].dataset.datePickerInit) continue;
                newEls[i].dataset.datePickerInit = '1';
                try { attachDatePickerBehavior(newEls[i]); } catch (_) {}
            }
        });
    }

    function attachDatePickerBehavior(wrapper) {
        var nativeInput = wrapper.querySelector('[data-date-picker-native]');
        var trigger = wrapper.querySelector('[data-date-picker-trigger]');
        var dlg = wrapper.querySelector('[data-date-picker-dialog]');

        if (!nativeInput || !trigger || !dlg) return;

        // No-JS fallback: if showModal not supported, leave native input functional.
        if (typeof dlg.showModal !== 'function') return;

        // Build dialog ARIA structure once.
        dlg.setAttribute('role', 'dialog');
        dlg.setAttribute('aria-modal', 'true');
        dlg.setAttribute('aria-label', 'Seleziona data');

        // State: current view month/year (not necessarily selected date).
        var today = new Date();
        var viewYear = today.getFullYear();
        var viewMonth = today.getMonth(); // 0-based

        // Parse native input value to initialize view.
        if (nativeInput.value) {
            var parts = nativeInput.value.split(/[T ]/)[0].split('-');
            if (parts.length >= 2) {
                viewYear = parseInt(parts[0], 10) || viewYear;
                viewMonth = (parseInt(parts[1], 10) - 1) || viewMonth;
            }
        }

        // Build dialog inner structure.
        dlg.innerHTML = '';

        // Header: prev button, month+year label, next button.
        var header = document.createElement('div');
        header.className = 'fjui-datepicker__header';

        var prevBtn = document.createElement('button');
        prevBtn.type = 'button';
        prevBtn.className = 'fjui-datepicker__nav-btn';
        prevBtn.setAttribute('aria-label', 'Mese precedente');
        prevBtn.textContent = '‹';

        var monthLabel = document.createElement('span');
        monthLabel.className = 'fjui-datepicker__month-label';

        var nextBtn = document.createElement('button');
        nextBtn.type = 'button';
        nextBtn.className = 'fjui-datepicker__nav-btn';
        nextBtn.setAttribute('aria-label', 'Mese successivo');
        nextBtn.textContent = '›';

        header.appendChild(prevBtn);
        header.appendChild(monthLabel);
        header.appendChild(nextBtn);
        dlg.appendChild(header);

        // Calendar grid container.
        var gridWrapper = document.createElement('div');
        gridWrapper.className = 'fjui-datepicker__grid';
        gridWrapper.setAttribute('role', 'grid');
        dlg.appendChild(gridWrapper);

        // Time section (shown when native input is datetime-local or time).
        var isDateTime = nativeInput.type === 'datetime-local' || nativeInput.type === 'time';
        var timeSection = null;
        var timeListbox = null;
        var timeEngine = null;

        if (isDateTime) {
            timeSection = document.createElement('div');
            timeSection.className = 'fjui-datepicker__time-section';

            var timeSectionLabel = document.createElement('div');
            timeSectionLabel.className = 'fjui-datepicker__time-label';
            timeSectionLabel.textContent = 'Orario';
            timeSection.appendChild(timeSectionLabel);

            timeListbox = document.createElement('ul');
            timeListbox.setAttribute('role', 'listbox');
            timeListbox.setAttribute('aria-label', 'Ora');
            timeListbox.className = 'fjui-datepicker__time-list';

            // 15-minute slots 06:00-22:00.
            var slotIdx = 0;
            for (var h = 6; h <= 22; h++) {
                for (var m = 0; m < 60; m += 15) {
                    if (h === 22 && m > 0) break;
                    var hStr = h < 10 ? '0' + h : '' + h;
                    var mStr = m < 10 ? '0' + m : '' + m;
                    var timeStr = hStr + ':' + mStr;
                    var li = document.createElement('li');
                    li.setAttribute('role', 'option');
                    li.id = 'fjui-time-opt-' + slotIdx;
                    li.setAttribute('aria-selected', 'false');
                    li.className = 'fjui-datepicker__time-option';
                    li.textContent = timeStr;
                    li.setAttribute('data-time', timeStr);
                    timeListbox.appendChild(li);
                    slotIdx++;
                }
            }

            // Dummy input for the listbox engine (time list is keyboard-navigable).
            var timeInput = document.createElement('input');
            timeInput.type = 'text';
            timeInput.className = 'sr-only';
            timeInput.setAttribute('aria-label', 'Seleziona orario');
            timeSection.appendChild(timeInput);
            timeSection.appendChild(timeListbox);
            dlg.appendChild(timeSection);

            timeEngine = createListboxEngine(timeInput, timeListbox, function(opt) {
                var t = opt.getAttribute('data-time');
                updateTimeSelection(t);
            });

            timeListbox.addEventListener('click', function(e) {
                var opt = e.target.closest('[role="option"]');
                if (opt) {
                    var t = opt.getAttribute('data-time');
                    updateTimeSelection(t);
                }
            });

            timeInput.addEventListener('keydown', function(e) {
                if (timeEngine) timeEngine.handleKey(e);
            });
        }

        // Footer: Conferma + Cancella buttons.
        var footer = document.createElement('div');
        footer.className = 'fjui-datepicker__footer';

        var clearBtn = document.createElement('button');
        clearBtn.type = 'button';
        clearBtn.className = 'fjui-datepicker__clear-btn';
        clearBtn.textContent = 'Cancella';

        var confirmBtn = document.createElement('button');
        confirmBtn.type = 'button';
        confirmBtn.className = 'fjui-datepicker__confirm-btn';
        confirmBtn.textContent = 'Conferma';

        footer.appendChild(clearBtn);
        footer.appendChild(confirmBtn);
        dlg.appendChild(footer);

        // Render the calendar grid for current viewYear/viewMonth.
        function renderGrid() {
            monthLabel.textContent = MONTH_NAMES_IT[viewMonth] + ' ' + viewYear;
            gridWrapper.setAttribute('aria-label', 'Calendario ' + MONTH_NAMES_IT[viewMonth] + ' ' + viewYear);
            gridWrapper.innerHTML = '';

            // Weekday header row (Monday-first).
            var headerRow = document.createElement('div');
            headerRow.setAttribute('role', 'row');
            for (var d = 0; d < DAYS_SHORT_IT.length; d++) {
                var dayHeader = document.createElement('div');
                dayHeader.setAttribute('role', 'columnheader');
                // Full Italian day names for aria-label.
                var fullDayNames = ['Lunedì','Martedì','Mercoledì','Giovedì','Venerdì','Sabato','Domenica'];
                dayHeader.setAttribute('aria-label', fullDayNames[d]);
                dayHeader.className = 'fjui-datepicker__weekday';
                dayHeader.textContent = DAYS_SHORT_IT[d];
                headerRow.appendChild(dayHeader);
            }
            gridWrapper.appendChild(headerRow);

            // Determine 1st of the month and its Monday-first weekday offset.
            var firstDay = new Date(viewYear, viewMonth, 1);
            // getDay(): 0=Sun,1=Mon,...,6=Sat. Convert to Monday-first: Mon=0,...,Sun=6.
            var startOffset = (firstDay.getDay() + 6) % 7;
            var daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();

            // Get selected date from native input.
            var selectedYear = -1, selectedMonth = -1, selectedDay = -1;
            if (nativeInput.value) {
                var sp = nativeInput.value.split(/[T ]/)[0].split('-');
                if (sp.length >= 3) {
                    selectedYear = parseInt(sp[0], 10);
                    selectedMonth = parseInt(sp[1], 10) - 1;
                    selectedDay = parseInt(sp[2], 10);
                }
            }

            var todayY = today.getFullYear();
            var todayM = today.getMonth();
            var todayD = today.getDate();

            // Build 6-row grid (max needed for any month layout).
            var day = 1;
            var row = null;
            var cellIdx = 0;

            for (var week = 0; week < 6; week++) {
                if (day > daysInMonth) break;
                row = document.createElement('div');
                row.setAttribute('role', 'row');

                for (var col = 0; col < 7; col++) {
                    var btn = document.createElement('button');
                    btn.setAttribute('role', 'gridcell');
                    btn.type = 'button';
                    btn.className = 'fjui-datepicker__day';

                    if ((week === 0 && col < startOffset) || day > daysInMonth) {
                        // Empty cell.
                        btn.setAttribute('aria-disabled', 'true');
                        btn.setAttribute('tabindex', '-1');
                        btn.textContent = '';
                        btn.className = 'fjui-datepicker__day fjui-datepicker__day--empty';
                    } else {
                        var isToday = (viewYear === todayY && viewMonth === todayM && day === todayD);
                        var isSelected = (viewYear === selectedYear && viewMonth === selectedMonth && day === selectedDay);

                        var ariaLabel = day + ' ' + MONTH_NAMES_IT[viewMonth] + ' ' + viewYear;
                        if (isToday) ariaLabel += ' (oggi)';
                        if (isSelected) ariaLabel += ' (selezionato)';

                        btn.setAttribute('aria-label', ariaLabel);
                        btn.setAttribute('aria-selected', isSelected ? 'true' : 'false');
                        btn.setAttribute('aria-disabled', 'false');
                        btn.setAttribute('tabindex', isSelected ? '0' : '-1');
                        btn.setAttribute('data-date-day', '' + day);
                        btn.textContent = '' + day;

                        if (isToday) btn.className = 'fjui-datepicker__day fjui-datepicker__day--today';
                        if (isSelected) btn.className = 'fjui-datepicker__day fjui-datepicker__day--selected';

                        (function(d) {
                            btn.addEventListener('click', function() { selectDay(d); });
                        })(day);

                        day++;
                    }
                    cellIdx++;
                    row.appendChild(btn);
                }
                gridWrapper.appendChild(row);

                // Advance day counter for empty leading cells in first row.
                if (week === 0 && day === 1) {
                    day = 1; // will be incremented in the loop above
                }
            }

            // Ensure at least one cell has tabindex=0 (roving tabindex).
            var focusable = gridWrapper.querySelector('[role="gridcell"][tabindex="0"]');
            if (!focusable) {
                var first = gridWrapper.querySelector('[role="gridcell"][data-date-day]');
                if (first) first.setAttribute('tabindex', '0');
            }
        }

        function selectDay(d) {
            var month = viewMonth + 1;
            var mStr = month < 10 ? '0' + month : '' + month;
            var dStr = d < 10 ? '0' + d : '' + d;
            var dateStr = viewYear + '-' + mStr + '-' + dStr;

            if (nativeInput.type === 'datetime-local') {
                // Preserve existing time component if present.
                var existingTime = '';
                if (nativeInput.value && nativeInput.value.indexOf('T') !== -1) {
                    existingTime = nativeInput.value.split('T')[1];
                }
                nativeInput.value = dateStr + 'T' + (existingTime || '00:00');
            } else {
                nativeInput.value = dateStr;
            }

            // Update trigger label with Italian display string.
            trigger.textContent = d + ' ' + MONTHS_IT[viewMonth] + ' ' + viewYear;

            // Re-render to update aria-selected state.
            renderGrid();
        }

        function updateTimeSelection(timeStr) {
            if (nativeInput.type === 'datetime-local') {
                var dateComponent = nativeInput.value
                    ? nativeInput.value.split('T')[0]
                    : '';
                if (dateComponent) {
                    nativeInput.value = dateComponent + 'T' + timeStr;
                }
            }
            // Mark selected option in the time listbox.
            var opts = timeListbox ? timeListbox.querySelectorAll('[role="option"]') : [];
            for (var i = 0; i < opts.length; i++) {
                opts[i].setAttribute('aria-selected', opts[i].getAttribute('data-time') === timeStr ? 'true' : 'false');
            }
        }

        // Arrow-key grid navigation.
        gridWrapper.addEventListener('keydown', function(e) {
            var focused = document.activeElement;
            if (!focused || !focused.hasAttribute('data-date-day')) return;
            var currentDay = parseInt(focused.getAttribute('data-date-day'), 10);
            if (isNaN(currentDay)) return;

            var delta = 0;
            if (e.key === 'ArrowLeft') { e.preventDefault(); delta = -1; }
            else if (e.key === 'ArrowRight') { e.preventDefault(); delta = 1; }
            else if (e.key === 'ArrowUp') { e.preventDefault(); delta = -7; }
            else if (e.key === 'ArrowDown') { e.preventDefault(); delta = 7; }
            else if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                selectDay(currentDay);
                return;
            } else {
                return;
            }

            var newDay = currentDay + delta;
            var daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();
            if (newDay < 1) {
                // Go to previous month.
                viewMonth--;
                if (viewMonth < 0) { viewMonth = 11; viewYear--; }
                renderGrid();
                var prevDays = new Date(viewYear, viewMonth + 1, 0).getDate();
                newDay = prevDays + newDay;
            } else if (newDay > daysInMonth) {
                // Go to next month.
                newDay = newDay - daysInMonth;
                viewMonth++;
                if (viewMonth > 11) { viewMonth = 0; viewYear++; }
                renderGrid();
            }

            // Move roving tabindex to new cell.
            var cells = gridWrapper.querySelectorAll('[role="gridcell"][data-date-day]');
            for (var i = 0; i < cells.length; i++) {
                cells[i].setAttribute('tabindex', '-1');
            }
            for (var j = 0; j < cells.length; j++) {
                if (parseInt(cells[j].getAttribute('data-date-day'), 10) === newDay) {
                    cells[j].setAttribute('tabindex', '0');
                    try { cells[j].focus(); } catch (_) {}
                    break;
                }
            }
        });

        // Prev/next month navigation.
        prevBtn.addEventListener('click', function() {
            viewMonth--;
            if (viewMonth < 0) { viewMonth = 11; viewYear--; }
            renderGrid();
        });

        nextBtn.addEventListener('click', function() {
            viewMonth++;
            if (viewMonth > 11) { viewMonth = 0; viewYear++; }
            renderGrid();
        });

        // Confirm button: close dialog.
        confirmBtn.addEventListener('click', function() {
            try { dlg.close(); } catch (_) {}
        });

        // Clear button: reset native input and trigger label.
        clearBtn.addEventListener('click', function() {
            nativeInput.value = '';
            trigger.textContent = 'Seleziona data';
            renderGrid();
            try { dlg.close(); } catch (_) {}
        });

        // Trigger click: capture return target, open dialog.
        trigger.addEventListener('click', function() {
            renderGrid();
            try { dlg.showModal(); } catch (_) {}
        });

        // Focus return: on dialog close, restore focus to trigger if browser
        // did not auto-return it (Pitfall 6).
        dlg.addEventListener('close', function() {
            if (document.activeElement === document.body) {
                try { trigger.focus(); } catch (_) {}
            }
        });

        // Initialize trigger label from existing native input value.
        if (nativeInput.value) {
            var iv = nativeInput.value.split(/[T ]/)[0].split('-');
            if (iv.length >= 3) {
                var iy = parseInt(iv[0], 10);
                var im = parseInt(iv[1], 10) - 1;
                var id2 = parseInt(iv[2], 10);
                trigger.textContent = id2 + ' ' + MONTHS_IT[im] + ' ' + iy;
            }
        }
    }
"#;

const isValidCalendarDate = (year: number, month: number, day: number): boolean => {
  const date = new Date(Date.UTC(year, month - 1, day));

  return (
    date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day
  );
};

export const parseImportDate = (value: string): string | undefined => {
  const iso = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);

  if (iso) {
    return isValidCalendarDate(Number(iso[1]), Number(iso[2]), Number(iso[3])) ? value : undefined;
  }

  const dayFirst = /^(\d{1,2})[-/](\d{1,2})[-/](\d{4})$/.exec(value);

  if (dayFirst) {
    const [, day, month, year] = dayFirst;

    if (!isValidCalendarDate(Number(year), Number(month), Number(day))) {
      return undefined;
    }

    return `${year}-${month.padStart(2, '0')}-${day.padStart(2, '0')}`;
  }

  return undefined;
};

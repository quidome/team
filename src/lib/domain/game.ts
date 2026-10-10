/** Returns no departure time while the start time or travel time is not known. */
export const suggestDepartureTime = (
  startTime: string | undefined,
  travelMinutes: number | undefined,
  arrivalBufferMinutes: number,
): string | undefined => {
  if (startTime === undefined || travelMinutes === undefined) {
    return undefined;
  }

  const [hours, minutes] = startTime.split(':').map(Number);
  const minutesSinceMidnight = hours * 60 + minutes - travelMinutes - arrivalBufferMinutes;
  const normalizedMinutes = ((minutesSinceMidnight % 1440) + 1440) % 1440;
  const departureHours = Math.floor(normalizedMinutes / 60);
  const departureMinutes = normalizedMinutes % 60;

  return `${String(departureHours).padStart(2, '0')}:${String(departureMinutes).padStart(2, '0')}`;
};

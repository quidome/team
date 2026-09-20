Feature: Review participation and duty history
  The coordinator can review current-season player summaries.

  Scenario: Viewing player participation percentages
    Given attendance records and completed duties exist for the active roster
    When the coordinator opens History
    Then game and training percentages are shown per player
    And completed duty counts are shown per player
    And recorded participation entries are listed

  Scenario: Inspecting audit history
    Given a coordinator import has completed
    When the coordinator opens History
    Then the import action, source, and result counts are visible in the audit list

  Scenario: Inspecting task changes
    Given a manual reminder is created and completed
    When the coordinator opens History
    Then the task creation and status change are visible in the audit list

  Scenario: Inspecting game lifecycle changes
    Given a game is scheduled, rescheduled, and cancelled
    When the coordinator opens History
    Then the game lifecycle actions are visible in the audit list

  Scenario: Inspecting attendance changes
    Given attendance is stored for a training or game occurrence
    When the coordinator opens History
    Then the attendance action and record counts are visible in the audit list

  Scenario: Inspecting duty coordination changes
    Given duty requirements, assignments, signups, and statuses are changed
    When the coordinator opens History
    Then the duty coordination actions are visible in the audit list

Feature: Attendance recording
  The coordinator records training attendance in bulk while retaining absence reasons.

  Scenario: Recording attendance with one absence
    Given Avery, Blake, and Casey are eligible for a training occurrence
    When the coordinator records Blake as absent because of illness
    Then Avery and Casey are stored as present
    And Blake is stored as absent with the illness reason

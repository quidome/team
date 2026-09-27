Feature: Club schedule sync
  The coordinator keeps the club's games in step with the federation schedule feed.
  Clubs, teams, games, and locations keep their own identity in the app and remember the source IDs they were matched to.
  The sync stores all games of all club teams. Every sync is a preview first; nothing changes until the coordinator confirms.
  The feed decides date, start time, teams, location, and result. The app decides travel time, arrival buffer,
  duties, assignments, attendance, and notes; the sync never changes those.

  Scenario: Importing the club's games for a season
    Given the club Blue Drakes is configured with source club ID 37
    And the feed lists 71 games for Blue Drakes in season 2026-2027
    When the coordinator syncs the club schedule
    Then 71 games are stored, each with its source game ID
    And each game has a home team, an away team, a date, a start time, and a location

  Scenario: Recognising the club's own teams
    Given the feed lists games for the Blue Drakes teams M16 1, M16 2, M18 1, MSE 1, X12 1, and X14 1
    When the coordinator syncs the club schedule
    Then each of those teams is stored as a team of the club
    And opponent teams are stored as teams of their own clubs

  Scenario: Keeping a team when its source team ID changes
    Given U16-1 is linked to source team ID 184 for the first half of the season
    And the feed lists a second-half game for Blue Drakes M16 1 with source team ID 912
    When the coordinator links source team ID 912 to U16-1
    Then first-half and second-half games both belong to U16-1
    And the players and duty history of U16-1 are unchanged

  Scenario: Previewing a sync before applying it
    Given the feed has 3 new games and 1 moved game compared to the stored games
    When the coordinator requests a sync preview
    Then the preview lists 3 new games and 1 moved game with the old and new date and time
    And no stored game has changed

  Scenario: Applying a confirmed sync
    Given the coordinator has reviewed a sync preview
    When the coordinator confirms the sync
    Then the changes from the preview are stored

  Scenario: Linking a game from an earlier import to the feed
    Given a game of Blue Drakes M16-1 against Archipel M16-1 on 2026-10-03 was imported from a CSV file
    And the feed lists Blue Drakes M16 1 against Archipel M16 1 on 2026-10-03 with source game ID 1073142
    When the coordinator requests a sync preview
    Then the preview proposes to link the stored game to source game ID 1073142
    And confirming the sync links the game without adding a duplicate

  Scenario: Updating a rescheduled game
    Given a synced game with source game ID 1073150 is scheduled on 2026-10-03 at 16:00
    And the feed now lists that game on 2026-10-10 at 15:00
    When the coordinator confirms the sync
    Then the original occurrence remains stored as cancelled
    And a new scheduled occurrence is stored for the same game

  Scenario: Moving duties and attendance with a rescheduled game
    Given a synced game has duty places, assigned players, and recorded attendance
    And the feed moves that game to another date
    When the coordinator confirms the sync
    Then the duty places, assigned players, and attendance belong to the new occurrence
    And the sync report lists the moved game so the coordinator can check the players

  Scenario: Letting the feed win over a local change
    Given the coordinator changed the start time of a synced game to 16:30
    And the feed lists that game at 16:00
    When the coordinator requests a sync preview
    Then the preview shows the start time change from 16:30 to 16:00
    And confirming the sync stores 16:00

  Scenario: Keeping app-owned game data during a sync
    Given a synced game has a travel time, an arrival buffer, and duty places
    And the feed changes the start time of that game
    When the coordinator confirms the sync
    Then the travel time, arrival buffer, and duty places are unchanged

  Scenario: Updating a result
    Given a synced game has no result
    And the feed lists the result 57 - 43 for that game
    When the coordinator confirms the sync
    Then the game has the result 57 - 43

  Scenario: Keeping games that are not in the feed
    Given a tournament game was entered by hand or imported from a CSV file
    And the feed does not list that game
    When the coordinator confirms the sync
    Then the game is unchanged and not marked as missing

  Scenario: Syncing twice does not duplicate games
    Given the club schedule was synced
    When the coordinator requests a sync preview again without feed changes
    Then the preview lists no changes

  Scenario: Flagging a game that disappeared from the feed
    Given a synced game is no longer listed in the feed
    When the coordinator confirms the sync
    Then the game is kept and marked as missing from the source
    And the coordinator decides whether to cancel it

  Scenario: Handling a game without a known start time
    Given the feed lists a game at 00:00
    When the coordinator syncs the club schedule
    Then the game is stored without a start time
    And no departure time is suggested for it

  Scenario: Ignoring placeholder results for unplayed games
    Given the feed lists an unplayed game with the result 0 - 0
    When the coordinator syncs the club schedule
    Then the game has no result

  Scenario: Storing teams with the feed name and an optional short name
    Given the feed lists the team Blue Drakes M16 1
    When the coordinator syncs the club schedule
    Then the team is stored with the feed name Blue Drakes M16 1
    And the coordinator can give it the short name U16-1
    And the app shows the short name when one is set

  Scenario: Deriving own teams from the own club
    Given Blue Drakes is the own club
    When the coordinator syncs the club schedule
    Then Blue Drakes teams count as own teams
    And teams of other clubs count as opponents

  Scenario: Storing a location without a known travel time
    Given the feed lists the location Sporthal Wolfskamer in Huizen with coordinates
    When the coordinator syncs the club schedule
    Then the location is stored with its address and coordinates
    And its travel time is not set
    And games at that location show that the travel time is not set

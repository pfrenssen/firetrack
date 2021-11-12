Feature: Expenses
  In order to keep track of my expenses
  As a user
  I want to enter, manage and view my expenses

  @javascript
  Scenario: Add an expense
    Given I am logged in as "pallas.park@email.gr"
    And I click "Expenses" in the "sidebar navigation"
    Then I should see the heading "Expenses"
    And I should be on "/expenses"
    When I click "Add expense"
    Then I should be on "/expenses/add"
    And I should see the heading "Add expense"
    And the "Amount" field should not contain a value
    And "Education" should be selected in the "Category" hierarchical dropdown
    And the "Category" hierarchical dropdown should not be expanded
    And the "Date" field should contain today's date

    When I fill in "Amount" with "99.95"
    And I select the "Groceries" option in the "Category" hierarchical dropdown
    And I fill in "Date" with "2020-02-21"
    And I press "Add"
    Then I should see the success message "Added €99.95 expense to the Internet category."
    And I should have 1 expense

  @javascript
  Scenario Outline: JS validation of correct input
    Given I am logged in as "rosa.mcfly@mymail.bg"
    And I am on the add expense form
    And I fill in "Amount" with "<amount>"
    And I fill in "Date" with "<date>"
    Then I should not see any form validation error messages
    Then I should not see "Please enter a valid date."

    Examples:
      | amount      | date       |
      | 1           | 2020-02-28 |
      | 1.01        | 2020-02-29 |
      | 10.00000    | 2021-01-01 |
      | 999         | 2021-04-30 |
      | 9999999.99  | 2021-12-31 |
      | 9999999     | 2021       |
      | 1234567.89  | 2021-01    |
      | 00009999999 | 2021-1     |
      | 0.01        | 2021-1-1   |
      | 0.01        | 19750207   |
      | 99.95       | 1900-01-01 |
      | 12.5        | 2099-12-31 |

  @javascript
  Scenario Outline: JS validation of incorrect input
    Given I am logged in as "rosa.mcfly@mymail.bg"
    And I am on the add expense form
    And I fill in "Amount" with "<amount>"
    And I fill in "Date" with "<date>"
    Then I should see 2 form validation error messages
    And I should see the form validation error "<amount error>"
    And I should see the form validation error "<date error>"

    Examples:
      | amount      | date       | amount error                            | date error                 |
      |             |            | Please enter an amount.                 | Please enter a valid date. |
      | 0           | 1899-12-31 | Amount should be 0.01 or greater.       | Please enter a valid date. |
      | 0.00        | 2100-01-01 | Amount should be 0.01 or greater.       | Please enter a valid date. |
      | 99999999.99 | invalid    | Amount should be 9999999.99 or smaller. | Please enter a valid date. |
      | 10000000.00 | 2021-02-29 | Amount should be 9999999.99 or smaller. | Please enter a valid date. |
      | 10000000    | 2021-13-01 | Amount should be 9999999.99 or smaller. | Please enter a valid date. |

Feature: User authentication
  In order to protect the integrity of the website
  As a product owner
  I want to make sure users can only access pages they are authorized to

  Scenario Outline: Anonymous user can access public pages
    When I go to "<path>"
    Then the response status code should be 200

    Examples:
      | path          |
      | /             |
      | user/login    |
      | user/register |

  Scenario Outline: Anonymous user cannot access private pages
    When I go to "<path>"
    Then the response status code should be 403

    Examples:
      | path         |
      | expenses     |
      | expenses/add |

// SPDX-License-Identifier: MIT
pragma solidity >=0.4.22 <0.9.0;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract UserRegistry is Initializable {
    struct Record {
        string endpoint;
    }

    mapping(address => Record) records;

    using EnumerableSet for EnumerableSet.AddressSet;
    EnumerableSet.AddressSet users;

    event UserJoin(address indexed user, Record record);
    event UserLeave(address indexed user);
    event UserRegistryUpdate(address indexed user, Record record);

    function initialize() public initializer {
        __UserRegistry_init_unchained();
    }

    function __UserRegistry_init() internal onlyInitializing {
        __UserRegistry_init_unchained();
    }

    function __UserRegistry_init_unchained() internal onlyInitializing {}

    function setRecord(Record memory record) public {
        if (!users.contains(msg.sender)) {
            emit UserJoin(msg.sender, record);
        }

        records[msg.sender] = record;

        emit UserRegistryUpdate(msg.sender, record);
    }

    function getRecord(address user) public view returns (Record memory) {
        return records[user];
    }

    function removeRecord() public {
        require(users.contains(msg.sender), "user must be valid");

        users.remove(msg.sender);
        delete records[msg.sender];

        emit UserLeave(msg.sender);
    }

    // **Very Expensive**
    function getUsers() public view returns (address[] memory) {
        return users.values();
    }
}

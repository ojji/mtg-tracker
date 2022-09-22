using System;
using System.Threading.Tasks;
using UnityEngine;
using Wizards.Mtga.FrontDoorModels;
using Newtonsoft.Json;
using System.Threading;
using System.Linq;

namespace mtga_datacollector
{
  public class CollectorEvent
  {
    public string Timestamp;
    public object Attachment;
  }

  public class AccountData
  {
    public string userId;
    public string screenName;
  }

  public class CardInventoryData
  {
    public uint grpId;
    public int count;
  }

  public class MtgaDataCollector : MonoBehaviour
  {
    private UnityCrossThreadLogger _logger = new UnityCrossThreadLogger("MTGADataCollector");
    private bool _subscribedToAccountInfo = false;
    private bool _subscribedToInventory = false;

    public void Start()
    {
      _logger.Info($"[initialization]Initialization started at {System.DateTime.Now:O}");
      Task initialize = new Task(Initialize);
      initialize.Start();
    }

    private void Initialize()
    {
      if (!_subscribedToAccountInfo)
      {
        SubscribeToAccountInfo();
      }
      if (!_subscribedToInventory)
      {
        CollectInventoryAndSubscribeToChanges();
      }
      if (!_subscribedToInventory || !_subscribedToAccountInfo)
      {
        _logger.Info($"[initialization]Waiting for everyone to load {System.DateTime.Now:O}");
        System.Threading.Thread.Sleep(5000);
        Initialize();
      }

      _logger.Info($"[initialization]Initialization is done at {System.DateTime.Now:O}. Ready to go!");
    }

    private void SubscribeToAccountInfo()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.AccountClient != null && WrapperController.Instance.AccountClient.AccountInformation != null && WrapperController.Instance.AccountClient.AccountInformation.AccountID != null)
      {
        try
        {
          WrapperController.Instance.AccountClient.LoginStateChanged += AccountClientLoginStateChanged;
          var accountInfo = new AccountData
          {
            userId = WrapperController.Instance.AccountClient.AccountInformation.AccountID,
            screenName = WrapperController.Instance.AccountClient.AccountInformation.DisplayName
          };

          var logEntry = new CollectorEvent {
            Attachment = accountInfo,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          _logger.Info($"[account-info]{JsonConvert.SerializeObject(logEntry)}");
          _subscribedToAccountInfo = true;
        }
        catch (Exception e)
        {
          _logger.Info($"[account-info-error]{JsonConvert.SerializeObject(e)}");
        }
      }
    }

    private void AccountClientLoginStateChanged(LoginState obj)
    {
      _logger.Info($"[loginstate]{obj}");
      _subscribedToAccountInfo = false;
      _subscribedToInventory = false;
      if (WrapperController.Instance != null && WrapperController.Instance.AccountClient != null && WrapperController.Instance.AccountClient.AccountInformation != null && WrapperController.Instance.AccountClient.AccountInformation.AccountID != null)
      {
        WrapperController.Instance.AccountClient.LoginStateChanged -= AccountClientLoginStateChanged;
      }
      Task.Run(Initialize);
    }

    private void CollectInventoryAndSubscribeToChanges()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Cards.Count > 0)
      {
        try
        {
          WrapperController.Instance.InventoryManager.UnsubscribeFromAll(this.UpdateInventory);
          WrapperController.Instance.InventoryManager.SubscribeToAll(this.UpdateInventory);
          _subscribedToInventory = true;

          Task.Run(PeriodicUpdater);
        }
        catch (Exception e)
        {
          _logger.Info($"[collection-error]{JsonConvert.SerializeObject(e)}");
        }
      }

    }

    private void PeriodicUpdater()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Inventory != null)
      {
        try
        {
          var collection = WrapperController.Instance.InventoryManager.Cards.Select(pair =>
          {
            return new CardInventoryData
            {
              grpId = pair.Key,
              count = pair.Value,
            };
          }).ToArray();

          var collectionEntry = new CollectorEvent
          {
            Attachment = collection,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          _logger.Info($"[collection]{JsonConvert.SerializeObject(collectionEntry)}");

          CollectorEvent inventory = new CollectorEvent
          {
            Attachment = WrapperController.Instance.InventoryManager.Inventory,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          _logger.Info($"[inventory]{JsonConvert.SerializeObject(inventory)}");
        }
        catch (Exception e)
        {
          _logger.Info($"[collection-error]{JsonConvert.SerializeObject(e)}");
        }
      }

      Thread.Sleep(TimeSpan.FromMinutes(30));
      PeriodicUpdater();
    }

    private void UpdateInventory(ClientInventoryUpdateReportItem payload)
    {
      CollectorEvent inventoryUpdate = new CollectorEvent
      {
        Timestamp = String.Format($"{DateTime.Now:O}"),
        Attachment = payload
      };

      _logger.Info($"[inventory-update]{JsonConvert.SerializeObject(inventoryUpdate)}");
    }

    public void OnDestroy()
    {
      _logger.Info($"[initialization]Shutting down at {System.DateTime.Now:O}. Bye!");
    }

    public void OnDisable()
    {
      _logger.Info($"[initialization]Disabled at {System.DateTime.Now:O}. Oops!");
    }

    public void OnApplicationQuit()
    {
      _logger.Info($"[initialization]App quit at {System.DateTime.Now:O}. Bye bye!");
    }
  }
}

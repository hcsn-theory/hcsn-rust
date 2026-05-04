import json
import glob
import os
import numpy as np
import pandas as pd
from sklearn.preprocessing import StandardScaler
from sklearn.decomposition import PCA
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import accuracy_score

def analyze_correlation_collapse(directory):
    files = glob.glob(os.path.join(directory, "hcsn_mechanisms_*.json"))
    if not files:
        print("No mechanism data found in", directory)
        return
        
    all_data = []
    for f in files:
        with open(f, 'r') as fp:
            all_data.extend(json.load(fp))
            
    df = pd.DataFrame(all_data)
    if len(df) == 0:
        print("Dataset is empty.")
        return
        
    print(f"Loaded {len(df)} mechanism records.")
    
    features = ['stability', 'coherence', 'suppression', 'memory']
    X = df[features]
    y = df['survived'].astype(int)
    
    # 1. Pearson Correlation Matrix
    print("\n--- Pearson Correlation Matrix ---")
    corr_matrix = X.corr()
    print(corr_matrix.round(3))
    
    # 2. PCA
    print("\n--- Principal Component Analysis (PCA) ---")
    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)
    
    pca = PCA()
    pca.fit(X_scaled)
    explained_variance = pca.explained_variance_ratio_
    
    print("Explained Variance by Component:")
    for i, var in enumerate(explained_variance):
        print(f"  PC{i+1}: {var:.3f} ({var*100:.1f}%)")
        
    if explained_variance[0] > 0.6:
        print("✅ CORRELATION COLLAPSE DETECTED: First Principal Component (PERSISTENCE) explains >60% of variance.")
    else:
        print("❌ INDEPENDENT MECHANISMS: Multiple components are required to explain the variance.")
        
    # 3. Predictive Redundancy Test
    print("\n--- Predictive Redundancy Test (Target: survival) ---")
    rf = RandomForestClassifier(n_estimators=100, random_state=42)
    rf.fit(X_scaled, y)
    
    # Baseline accuracy
    y_pred_baseline = rf.predict(X_scaled)
    acc_baseline = accuracy_score(y, y_pred_baseline)
    print(f"Baseline Accuracy (all 5 variables): {acc_baseline:.3f}")
    
    for i, feat in enumerate(features):
        # Remove feature i
        X_reduced = np.delete(X_scaled, i, axis=1)
        
        rf_reduced = RandomForestClassifier(n_estimators=100, random_state=42)
        rf_reduced.fit(X_reduced, y)
        y_pred = rf_reduced.predict(X_reduced)
        acc_reduced = accuracy_score(y, y_pred)
        
        drop = acc_baseline - acc_reduced
        print(f"Accuracy drop when removing '{feat}': {drop:.4f}")
        
        if drop < 0.05:
            print(f"  -> '{feat}' is REDUNDANT (drop < 5%)")
        elif drop > 0.20:
            print(f"  -> '{feat}' is CRITICAL (drop > 20%)")
        else:
            print(f"  -> '{feat}' provides some independent signal")

if __name__ == "__main__":
    analyze_correlation_collapse("exports/conservation/patched")
